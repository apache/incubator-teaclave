use std::{
    any::Any,
    borrow::Cow,
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor, Deref},
    sync::Arc,
};

use policy_carrying_data::{
    field::{new_empty, new_null, Field, FieldDataArray, FieldDataRef, FieldRef},
    group::GroupsProxy,
    schema::{Schema, SchemaRef},
    Comparator, DataFrame,
};
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::{AAggExpr, AExpr, BinaryOp, Expr, GroupByMethod, Node},
    types::*,
};
use serde::{Deserialize, Serialize};

use crate::{
    context::{AggState, AggregationContext},
    executor::{ExecutionState, ExprArena},
    plan::expr_to_aexpr,
    udf::UserDefinedFunction,
};

use super::{aexpr_to_expr, ApplyOption};

pub type PhysicalExprRef = Arc<dyn PhysicalExpr>;

macro_rules! check_null_prop {
    ($ac:ident, $name:expr, $groups:expr) => {
        match &$ac.state {
            AggState::AggregatedFlat(_) => {
                if $ac.null_propagated {
                    let mut agg_s = $ac.aggregated();
                    let s = Arc::get_mut(&mut agg_s).unwrap();
                    s.rename($name.as_str())?;
                    return Ok(AggregationContext::new(agg_s, Cow::Borrowed($groups), true));
                } else {
                    return Err(PolicyCarryingError::ImpossibleOperation(
                        "cannog aggregate on an aggregated column".into(),
                    ));
                }
            }
            // Skipped.
            _ => (),
        }
    };
}

/// A physical expression trait.
#[typetag::serde(tag = "physical_expr")]
pub trait PhysicalExpr: Send + Sync + Debug {
    /// Downcasts to any.
    fn as_any_ref(&self) -> &dyn Any;

    /// Reveals the inner expression.
    fn expr(&self) -> Option<&Expr>;

    /// Evaluates this physical expression on a dataframe.
    fn evaluate(
        &self,
        _df: &DataFrame,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        Err(PolicyCarryingError::OperationNotSupported(
            "`evaluate` not supported".into(),
        ))
    }

    /// Evaluate on groups due to `group_by()`; aggregations can be applied on both groups and a single column.
    fn evaluate_groups<'a>(
        &self,
        _df: &DataFrame,
        _groups: &'a GroupsProxy,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        Err(PolicyCarryingError::OperationNotSupported(
            "`evaluate_groups` not supported".into(),
        ))
    }

    /// Returns the children of this node, if any.
    fn children(&self) -> Vec<PhysicalExprRef>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiteralExpr {
    pub(crate) literal: Box<dyn PrimitiveDataType>,
    pub(crate) expr: Expr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryOpExpr {
    pub(crate) left: PhysicalExprRef,
    pub(crate) op: BinaryOp,
    pub(crate) right: PhysicalExprRef,
    pub(crate) expr: Expr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilterExpr {
    pub(crate) input: PhysicalExprRef,
    pub(crate) by: PhysicalExprRef,
    pub(crate) expr: Expr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColumnExpr {
    pub(crate) name: String,
    pub(crate) expr: Expr,
    pub(crate) schema: Option<SchemaRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateExpr {
    pub(crate) input: PhysicalExprRef,
    pub(crate) agg_type: GroupByMethod,
    pub(crate) field: Option<Field>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyExpr {
    pub(crate) inputs: Vec<PhysicalExprRef>,
    pub(crate) function: Arc<dyn UserDefinedFunction>,
    pub(crate) expr: Expr,
    pub(crate) collect_groups: ApplyOption,
    pub(crate) allow_rename: bool,
    pub(crate) pass_name_to_apply: bool,
    pub(crate) input_schema: Option<SchemaRef>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CountExpr {
    pub(crate) expr: Expr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasExpr {
    pub(crate) from: PhysicalExprRef,
    pub(crate) to: String,
    pub(crate) expr: Expr,
}

#[typetag::serde]
impl PhysicalExpr for FilterExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![self.input.clone(), self.by.clone()]
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        let data = self.input.evaluate(df, state)?;
        let predicate = self.by.evaluate(df, state)?;

        data.filter(predicate.as_boolean()?)
    }
}

#[typetag::serde]
impl PhysicalExpr for BinaryOpExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![self.left.clone(), self.right.clone()]
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        let lhs = self.left.evaluate(df, state)?;
        let rhs = self.right.evaluate(df, state)?;

        if lhs.len() != rhs.len() {
            if lhs.len() != 1 && rhs.len() != 1 {
                return Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "evaluating binary op but the lengths are incorrect: lhs = {}, rhs = {}",
                    lhs.len(),
                    rhs.len()
                )));
            }
        }

        match self.op {
            // `as_ref()` is called because we cannot implement `Add` for Arc<dyn trait> since Arc is not
            // defined in our current crate.
            BinaryOp::Add => Ok(lhs.as_ref() + rhs.as_ref()),
            BinaryOp::Sub => Ok(lhs.as_ref() - rhs.as_ref()),
            BinaryOp::Mul => Ok(lhs.as_ref() * rhs.as_ref()),
            BinaryOp::Div => Ok(lhs.as_ref() / rhs.as_ref()),
            op => apply_binary_operator(lhs, rhs, op),
        }
    }
}

#[typetag::serde]
impl PhysicalExpr for LiteralExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![]
    }

    fn evaluate(
        &self,
        _df: &DataFrame,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        let data_type = self.literal.data_type();

        match data_type {
            DataType::UInt8 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<u8>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::UInt16 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<u16>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::UInt32 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<u32>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::UInt64 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<u64>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Int8 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<i8>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Int16 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<u16>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Int32 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<i32>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Int64 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<i64>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Float32 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<f32>().unwrap(),
                1,
                "literal".into(),
            ))),
            DataType::Float64 => Ok(Arc::new(FieldDataArray::new_with_duplicate(
                self.literal.try_cast::<f64>().unwrap(),
                1,
                "literal".into(),
            ))),

            _ => unimplemented!(),
        }
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }
}

#[typetag::serde]
impl PhysicalExpr for ColumnExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![]
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        match self.schema {
            None => df.find_column(self.name.as_ref()),

            // We proceed with searching in the schema.
            Some(ref schema) => {
                match schema
                    .fields_owned()
                    .into_iter()
                    .find(|col| col.name == *self.name)
                {
                    Some(column) => match df.find_column(column.name.as_ref()) {
                        Ok(column) => Ok(column.clone()),

                        // Not found in the dataframe but found in the current schema.
                        // We need to consult the state if there is something altered during the query.
                        Err(_) => match state.schema_cache.read().unwrap().as_ref() {
                            Some(schema) => todo!("{schema:?}"),
                            None => df.find_column(self.name.as_ref()),
                        },
                    },
                    None => df.find_column(self.name.as_ref()),
                }
            }
        }
    }

    fn evaluate_groups<'a>(
        &self,
        df: &DataFrame,
        groups: &'a GroupsProxy,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        println!("performing evaluate_groups for `ColumnExpr`");

        let s = self.evaluate(df, state)?;
        Ok(AggregationContext::new(s, Cow::Borrowed(groups), false))
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }
}

#[typetag::serde]
impl PhysicalExpr for AggregateExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![self.input.clone()]
    }

    fn evaluate_groups<'a>(
        &self,
        df: &DataFrame,
        groups: &'a GroupsProxy,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        println!("performing evaluate_groups for `AggregateExpr`");

        let mut ac = self.input.evaluate_groups(df, groups, state)?;
        let name = ac.data().name().to_string();
        println!("{ac:?}");

        if matches!(ac.state, AggState::Literal(_)) {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot aggregate literal types".into(),
            ))
        } else {
            check_null_prop!(ac, name, groups);
            let (d, groups_cur) = ac.get_final_aggregation();
            let aggregated = d.aggregate(self.agg_type, &groups_cur)?;

            Ok(AggregationContext::new(
                aggregated,
                Cow::Borrowed(groups),
                true,
            ))
        }
    }

    fn expr(&self) -> Option<&Expr> {
        None
    }
}

#[typetag::serde]
impl PhysicalExpr for ApplyExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        self.inputs.clone()
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        let mut inputs = self
            .inputs
            .iter()
            .map(|expr| expr.evaluate(df, state))
            .collect::<PolicyCarryingResult<Vec<_>>>()?;

        Ok(self.function.call(&mut inputs)?.unwrap_or_else(|| {
            // If `to_field` failed here, there is nothing we can do to recover it from error state.
            // So we simply panic if error occurs.
            let field = self.to_field(self.input_schema.as_ref().unwrap()).unwrap();
            Arc::from(new_null(field, 1))
        }))
    }

    fn evaluate_groups<'a>(
        &self,
        _df: &DataFrame,
        _groups: &'a GroupsProxy,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        todo!()
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }
}

#[typetag::serde]
impl PhysicalExpr for CountExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![]
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        Ok(Arc::new(FieldDataArray::new(
            Field::new(
                "COUNT(*)".into(),
                DataType::UInt64,
                false,
                Default::default(),
            )
            .into(),
            vec![df.shape().1 as u64],
        )))
    }

    fn evaluate_groups<'a>(
        &self,
        _df: &DataFrame,
        groups: &'a GroupsProxy,
        _state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        Ok(AggregationContext::new(
            groups.row_count()?,
            Cow::Borrowed(groups),
            true,
        ))
    }
}

#[typetag::serde]
impl PhysicalExpr for AliasExpr {
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn children(&self) -> Vec<PhysicalExprRef> {
        vec![self.from.clone()]
    }

    fn expr(&self) -> Option<&Expr> {
        Some(&self.expr)
    }

    fn evaluate(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<FieldDataRef> {
        let mut df = self.from.evaluate(df, state)?;
        let df_mut = Arc::get_mut(&mut df).unwrap();
        df_mut.rename(&self.to)?;

        Ok(df)
    }

    fn evaluate_groups<'a>(
        &self,
        df: &DataFrame,
        groups: &'a GroupsProxy,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<AggregationContext<'a>> {
        let mut ac = self.from.evaluate_groups(df, groups, state)?;
        ac.with_func(|df| {
            let df_mut = Arc::get_mut(df).ok_or(PolicyCarryingError::OperationNotAllowed(
                "cannot get `Arc` because it is shared".into(),
            ))?;
            df_mut.rename(&self.to)
        })?;

        Ok(ac)
    }
}

impl ApplyExpr {
    /// Given a schema, this function extracts the field from the schema.
    pub fn to_field(&self, schema: &SchemaRef) -> PolicyCarryingResult<FieldRef> {
        expr_to_field(&self.expr, schema, false)
    }
}

/// Applies the binary operator on the two [`FieldDataRef`] and returns the result.
pub(crate) fn apply_binary_operator(
    lhs: FieldDataRef,
    rhs: FieldDataRef,
    op: BinaryOp,
) -> PolicyCarryingResult<FieldDataRef> {
    match op {
        // We still match `add`, `sub`, etc. because this function may be called in other contexts.
        BinaryOp::Add => Ok(lhs.as_ref() + rhs.as_ref()),
        BinaryOp::Sub => Ok(lhs.as_ref() - rhs.as_ref()),
        BinaryOp::Mul => Ok(lhs.as_ref() * rhs.as_ref()),
        BinaryOp::Div => Ok(lhs.as_ref() / rhs.as_ref()),
        BinaryOp::Gt => Ok(Arc::new(lhs.as_ref().gt_bool(&rhs.as_ref())?)),
        BinaryOp::Ge => Ok(Arc::new(lhs.as_ref().ge_bool(&rhs.as_ref())?)),
        BinaryOp::Lt => Ok(Arc::new(lhs.as_ref().lt_bool(&rhs.as_ref())?)),
        BinaryOp::Le => Ok(Arc::new(lhs.as_ref().le_bool(&rhs.as_ref())?)),
        BinaryOp::Eq => Ok(Arc::new(lhs.as_ref().eq_bool(&rhs.as_ref())?)),
        BinaryOp::Ne => Ok(Arc::new(lhs.as_ref().ne_bool(&rhs.as_ref())?)),

        // Logical connectors: evaluate lhs and rhs and do logical evaluation on the both sides (should be applied
        // directly on boolean data field).
        BinaryOp::And => match (lhs.try_cast::<bool>(), rhs.try_cast::<bool>()) {
            (Ok(lhs), Ok(rhs)) => Ok(Arc::new(lhs.bitand(rhs)?)),
            (_, _) => Err(PolicyCarryingError::ImpossibleOperation(
                "cannot evaluate `&` on non-boolean arrays.".into(),
            )),
        },
        BinaryOp::Or => match (lhs.try_cast::<bool>(), rhs.try_cast::<bool>()) {
            (Ok(lhs), Ok(rhs)) => Ok(Arc::new(lhs.bitor(rhs)?)),
            (_, _) => Err(PolicyCarryingError::ImpossibleOperation(
                "cannot evaluate `|` on non-boolean arrays.".into(),
            )),
        },
        BinaryOp::Xor => match (lhs.try_cast::<bool>(), rhs.try_cast::<bool>()) {
            (Ok(lhs), Ok(rhs)) => Ok(Arc::new(lhs.bitxor(rhs)?)),
            (_, _) => Err(PolicyCarryingError::ImpossibleOperation(
                "cannot evaluate `^` on non-boolean arrays.".into(),
            )),
        },
    }
}

/// Extracts the [`Field`] information from the arena-ed expression which may be evaluated on a given column.
/// TODO: Implement this.
pub(crate) fn get_field_from_aexpr(
    aexpr: &AExpr,
    expr_arena: &ExprArena,
    schema: SchemaRef,
) -> PolicyCarryingResult<Field> {
    match aexpr {
        AExpr::Agg(aggregation) => match aggregation {
            AAggExpr::Max { input, .. }
            | AAggExpr::Min { input, .. }
            | AAggExpr::Sum(input)
            | AAggExpr::Mean(input) => {
                get_field_from_aexpr(expr_arena.get(*input), expr_arena, schema)
            }
        },
        AExpr::Column(name) => schema
            .fields()
            .iter()
            .find(|column| &column.name == name)
            .map(|inner| inner.deref().clone())
            .ok_or(PolicyCarryingError::ColumnNotFound(name.clone())),
        // Alias: take the type of the expression input and propagate it to the output field.
        AExpr::Alias(input, name) => {
            let original =
                get_field_from_aexpr(expr_arena.get(*input), expr_arena, schema.clone())?;

            Ok(Field::new(
                name.clone(),
                original.data_type,
                false,
                Default::default(),
            ))
        }
        AExpr::Filter { input, .. } => {
            get_field_from_aexpr(expr_arena.get(*input), expr_arena, schema.clone())
        }
        AExpr::Literal(literal) => Ok(Field::new_literal(literal.data_type())),
        AExpr::BinaryOp { left, op, .. } => match op {
            BinaryOp::Eq
            | BinaryOp::Ge
            | BinaryOp::Gt
            | BinaryOp::Le
            | BinaryOp::Lt
            | BinaryOp::Ne
            | BinaryOp::Or => Ok(Field::new(
                get_field_from_aexpr(expr_arena.get(*left), expr_arena, schema)?.name,
                DataType::Boolean,
                false,
                Default::default(),
            )),

            // For arithmetic binary expression it is a little bit complex because there are different types.
            _ => unimplemented!(),
        },
        _ => panic!("should not go here"),
    }
}

/// Converts this arena-ed aggregation expression into a physical expression [`PhysicalExpr`].
pub(crate) fn make_physical_expr_aaggexpr(
    parent: Node,
    aexpr: AAggExpr,
    expr_arena: &mut ExprArena,
    schema: Option<SchemaRef>,
    state: &ExecutionState,
    // Discern `in_aggregation`.
    in_aggregation: bool,
) -> PolicyCarryingResult<PhysicalExprRef> {
    log::debug!("{parent:?}, {aexpr:?}, {schema:?}, {state:?}, {in_aggregation}");

    let input = make_physical_expr(
        aexpr.get_input().clone(),
        expr_arena,
        schema.clone(),
        state,
        in_aggregation,
    )?;

    match in_aggregation {
        // We are not in an aggregation context, so we need to manually create the function that applies to the final result.
        // This typically occurs in a select/projection context.
        // FIXME: To unify the behavior of queries w or w/o `groupby` clause, we force the latter to `groupby None`.
        false => todo!(),
        // We are already in an aggregation context.
        true => Ok(Arc::new(AggregateExpr {
            input,
            agg_type: aexpr.into(),
            field: schema
                .map(|schema| get_field_from_aexpr(expr_arena.get(parent), expr_arena, schema))
                .transpose()?,
        })),
    }
}

/// Converts this arena-ed expression into a physical expression [`PhysicalExpr`].
pub(crate) fn make_physical_expr(
    aexpr: Node,
    expr_arena: &mut ExprArena,
    schema: Option<SchemaRef>,
    state: &ExecutionState,
    in_aggregation: bool, // Are we doing this operation in an aggregation context?
) -> PolicyCarryingResult<PhysicalExprRef> {
    let expr = expr_arena.get(aexpr).clone();

    match expr {
        AExpr::Alias(from, to) => {
            let from = make_physical_expr(from, expr_arena, schema, state, in_aggregation)?;
            Ok(Arc::new(AliasExpr {
                from,
                to,
                expr: aexpr_to_expr(aexpr, expr_arena),
            }))
        }
        AExpr::Column(name) => Ok(Arc::new(ColumnExpr {
            name,
            expr: aexpr_to_expr(aexpr, expr_arena),
            schema,
        })),
        AExpr::Count => Ok(Arc::new(CountExpr { expr: Expr::Count })),
        AExpr::Literal(literal) => Ok(Arc::new(LiteralExpr {
            literal,
            expr: aexpr_to_expr(aexpr, expr_arena),
        })),
        AExpr::BinaryOp { left, op, right } => {
            let left = make_physical_expr(left, expr_arena, schema.clone(), state, in_aggregation)?;
            let right = make_physical_expr(right, expr_arena, schema, state, in_aggregation)?;
            Ok(Arc::new(BinaryOpExpr {
                left,
                right,
                op,
                expr: aexpr_to_expr(aexpr, expr_arena),
            }))
        }
        AExpr::Filter { input, by } => {
            let input =
                make_physical_expr(input, expr_arena, schema.clone(), state, in_aggregation)?;
            let by = make_physical_expr(by, expr_arena, schema, state, in_aggregation)?;

            Ok(Arc::new(FilterExpr {
                input,
                by,
                expr: aexpr_to_expr(aexpr, expr_arena),
            }))
        }
        AExpr::Agg(aggregation) => make_physical_expr_aaggexpr(
            aexpr,
            aggregation,
            expr_arena,
            schema,
            state,
            in_aggregation,
        ),
        AExpr::Wildcard => panic!("wildcard should be handled at higher levels"),
    }
}

/// Extracts the field information from a given arena-ed expression, which is useful for soem fallback operations.
pub(crate) fn aexpr_to_field(
    aexpr: &AExpr,
    expr_arena: &ExprArena,
    schema: &SchemaRef,
    in_aggregation: bool,
) -> PolicyCarryingResult<FieldRef> {
    match aexpr {
        AExpr::Column(column_name) => {
            match schema
                .fields()
                .into_iter()
                .find(|field| &field.name == column_name)
            {
                Some(field) => Ok(field.clone()),
                None => Err(PolicyCarryingError::ColumnNotFound(column_name.clone())),
            }
        }
        AExpr::Alias(from, to) => {
            let field = aexpr_to_field(expr_arena.get(*from), expr_arena, schema, in_aggregation)?;
            Ok(Arc::new(Field::new(
                to.clone(),
                field.data_type,
                field.nullable,
                field.metadata.clone(),
            )))
        }
        AExpr::Literal(val) => Ok(Arc::new(Field::new(
            "literal".into(),
            val.data_type(),
            true,
            Default::default(),
        ))),
        // This has some potential problem: what if the real column is on the right side?
        AExpr::BinaryOp { left, .. } => {
            aexpr_to_field(expr_arena.get(*left), expr_arena, schema, in_aggregation)
        }
        AExpr::Filter { input, .. } => {
            aexpr_to_field(expr_arena.get(*input), expr_arena, schema, in_aggregation)
        }
        AExpr::Agg(agg) => match agg {
            AAggExpr::Max { input, .. } => {
                aexpr_to_field(expr_arena.get(*input), expr_arena, schema, in_aggregation)
            }
            AAggExpr::Min { input, .. } => {
                aexpr_to_field(expr_arena.get(*input), expr_arena, schema, in_aggregation)
            }
            AAggExpr::Mean(input) => {
                aexpr_to_field(expr_arena.get(*input), expr_arena, schema, in_aggregation)
            }
            AAggExpr::Sum(input) => {
                aexpr_to_field(expr_arena.get(*input), expr_arena, schema, in_aggregation)
            }
        },
        AExpr::Count => Ok(Arc::new(Field::new(
            "COUNT(*)".into(),
            DataType::UInt64,
            false,
            Default::default(),
        ))),

        // Cannot extract field information from this type because it should be expaned at higher level!
        expr => Err(PolicyCarryingError::InvalidInput(format!(
            "{expr:?} should not be evaluated at this level"
        ))),
    }
}

/// Extracts the field information from a given expression, which is useful for soem fallback operations.
pub(crate) fn expr_to_field(
    expr: &Expr,
    schema: &SchemaRef,
    in_aggregation: bool,
) -> PolicyCarryingResult<FieldRef> {
    fn expr_to_field_impl(
        expr_arena: &mut ExprArena,
        expr: &Expr,
        schema: &SchemaRef,
        in_aggregation: bool,
    ) -> PolicyCarryingResult<FieldRef> {
        let root = expr_to_aexpr(expr.clone(), expr_arena)?;
        let root = expr_arena.get(root);
        aexpr_to_field(root, expr_arena, schema, in_aggregation)
    }

    // A temporary arena for visiting the expression tree.
    let mut expr_arena = ExprArena::with_capacity(8);
    expr_to_field_impl(&mut expr_arena, expr, schema, in_aggregation)
}

#[allow(unused)]
fn replace_with_empty(dst: &mut FieldDataRef) -> FieldDataRef {
    let field = Field::new(
        dst.name().into(),
        DataType::Int64,
        false,
        Default::default(),
    );
    let src = Arc::from(new_empty(field.into()));

    std::mem::replace(dst, src)
}

/// Take a list of expressions and a schema and determine the output schema.
pub(crate) fn expressions_to_schema(
    expr: &[Expr],
    schema: &SchemaRef,
    in_aggregation: bool,
) -> PolicyCarryingResult<Schema> {
    let fields = expr
        .iter()
        .map(|expr| expr_to_field(expr, schema, in_aggregation))
        .collect::<PolicyCarryingResult<Vec<_>>>()?;

    Ok(Schema::new(fields, Default::default()))
}

/// Extracts the column name from the given expression.
pub(crate) fn expr_to_name(expr: &Expr) -> PolicyCarryingResult<String> {
    for e in expr.into_iter() {
        match e {
            Expr::Column(name) | Expr::Alias { name, .. } => return Ok(name.clone()),
            Expr::Wildcard => {
                return Err(PolicyCarryingError::InvalidInput(
                    "cannot determine the output name from wildcard without a specific context"
                        .into(),
                ))
            }
            _ => continue,
        }
    }

    Err(PolicyCarryingError::InvalidInput(format!(
        "unable to find root column name for expr '{expr:?}' when extracting from it"
    )))
}
