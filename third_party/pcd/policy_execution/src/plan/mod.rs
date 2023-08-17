use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
    ops::Deref,
    sync::Arc,
};

use bitflags::bitflags;
use hashbrown::HashSet;
use policy_carrying_data::{
    schema::{SchemaBuilder, SchemaRef},
    DataFrame,
};
use policy_core::{
    args,
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::{AAggExpr, AExpr, Aggregation, DistinctOptions, Expr, Node},
    pcd_ensures,
    policy::Policy,
    types::{ExecutorRefId, ExecutorType, JoinType},
};
use serde::{Deserialize, Serialize};

use crate::{
    executor::{
        create_executor, ExecutionState, ExecutorHandle, ExprArena, LogicalPlanArena,
        EXPR_ARENA_SIZE, LP_ARENA_SIZE,
    },
    plan::physical_expr::{expr_to_field, expr_to_name, expressions_to_schema, make_physical_expr},
    udf::UserDefinedFunction,
};

pub mod physical_expr;

pub type PhysicalPlan = (ExecutionState, ExecutorHandle);

bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct OptFlag: u32 {
        /// No optimizations.
        const NONE = 0;
        /// Pushes down the `where` clause.
        const PREDICATE_PUSHDOWN = 1 << 0;
        /// Pushes down the `projection` clause.
        const PROJECTION_PUSHDOWN = 1 << 1;
        /// Constant folding or expression simplication.
        const EXPR_SIMPLIFY = 1 << 2;
    }
}

macro_rules! delayed_err {
    ($result:expr, $inner:expr) => {
        match $result {
            Ok(data) => data,
            Err(err) => {
                return PlanBuilder {
                    plan: LogicalPlan::StagedError {
                        input: Some(Box::new($inner)),
                        err,
                    },
                }
            }
        }
    };
}

/// Basically, all the operations ona data frame should be categoried into a [`LogicalPlan`] that is
///
/// * A projection.
/// * A selection.
/// * An aggreation.
/// * Combinations of the above operations.
///
/// FIXME: Where do we put the policy? Probably not here.
#[derive(Clone)]
pub enum LogicalPlan {
    /// Select with *filter conditions* that work on a [`LogicalPlan`].
    Select {
        input: Box<LogicalPlan>,
        predicate: Expr,
        policy: Option<Box<dyn Policy>>,
    },

    /// The distinct expression.
    Distinct {
        input: Box<LogicalPlan>,
        /// How is this expression performed.
        options: DistinctOptions,
    },

    /// Projection
    Projection {
        input: Box<LogicalPlan>,
        /// Column 'names' as we may apply some transformation on columns.
        expression: Vec<Expr>,
        schema: SchemaRef,
        policy: Option<Box<dyn Policy>>,
    },

    /// Aggregate and group by
    Aggregation {
        input: Box<LogicalPlan>,
        schema: SchemaRef,
        /// Group by `keys`.
        keys: Arc<Vec<Expr>>,
        aggs: Vec<Expr>,
        apply: Option<Arc<dyn UserDefinedFunction>>,
        maintain_order: bool,
        policy: Option<Box<dyn Policy>>,
    },

    /// Join operation
    Join {
        input_left: Box<LogicalPlan>,
        input_right: Box<LogicalPlan>,
        schema: SchemaRef,
        left_on: Vec<Expr>,
        right_on: Vec<Expr>,
        options: JoinType,
    },

    /// Error that should be emitted later.
    StagedError {
        input: Option<Box<LogicalPlan>>,
        err: PolicyCarryingError,
        // Should we add a span?
    },

    DataFrameScan {
        #[cfg(feature = "built-in")]
        df: Option<Arc<DataFrame>>,
        schema: SchemaRef,
        // schema of the projected file
        output_schema: Option<SchemaRef>,
        projection: Option<Arc<Vec<String>>>,
        selection: Option<Expr>,
    },
}

impl Debug for LogicalPlan {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_impl(f, 0)
    }
}

impl LogicalPlan {
    fn fmt_impl(&self, f: &mut Formatter<'_>, indent: usize) -> std::fmt::Result {
        if indent != 0 {
            writeln!(f)?;
        }

        let sub_indent = indent + 4;
        match self {
            Self::Select {
                input, predicate, ..
            } => {
                write!(f, "{:indent$}FILTER {predicate:?} FROM", "")?;
                input.fmt_impl(f, indent)
            }
            Self::Projection {
                input, expression, ..
            } => {
                write!(f, "{:indent$} SELECT {expression:?} FROM", "")?;
                input.fmt_impl(f, sub_indent)
            }
            Self::Distinct { input, options } => {
                write!(
                    f,
                    "{:indent$}DISTINCT({})",
                    "",
                    match options.selected_columns {
                        Some(ref vec) => format!("{vec:?}"),
                        None => "*".into(),
                    }
                )?;
                input.fmt_impl(f, sub_indent)
            }
            Self::DataFrameScan {
                schema,
                projection,
                selection,
                ..
            } => {
                let total_columns = schema.fields_owned().len();
                let mut n_columns = "*".to_string();
                if let Some(columns) = projection {
                    n_columns = format!("{}", columns.len());
                }
                let selection = match selection {
                    Some(s) => Cow::Owned(format!("{s:?}")),
                    None => Cow::Borrowed("None"),
                };
                write!(
                    f,
                    "{:indent$}DF {:?}; PROJECT {}/{} COLUMNS; SELECTION: {:?}",
                    "",
                    schema
                        .fields_owned()
                        .into_iter()
                        .map(|field| field.name.clone())
                        .take(4)
                        .collect::<Vec<_>>(),
                    n_columns,
                    total_columns,
                    selection,
                )
            }
            Self::StagedError { input, err } => {
                write!(
                    f,
                    "Error occurred when constructing the logical plan: {err:?}\nThe previous output is {input:?}"
                )
            }
            Self::Aggregation {
                input, keys, aggs, ..
            } => {
                write!(f, "{:indent$}AGGREGATE", "")?;
                write!(f, "\n{:indent$}\t{aggs:?} GROUP BY {keys:?} FROM", "")?;
                write!(f, "\n{:indent$}\t{input:?}", "")
            }
            Self::Join {
                input_left,
                input_right,
                schema,
                left_on,
                right_on,
                options,
            } => {
                let fields = schema.fields();
                write!(f, "{:indent$}{options:?} JOIN:", "")?;
                write!(f, "\n{:indent$}LEFT ON: {left_on:?}", "")?;
                write!(f, "\n{:sub_indent$}{input_left:?}", "")?;
                write!(f, "\n{:indent$}RIGHT ON: {right_on:?}", "")?;
                write!(f, "\n{:sub_indent$}{input_right:?}", "")?;
                write!(
                    f,
                    "\n{:indent$}OUTPUT SCHEMA:\n{:sub_indent$}{fields:?}",
                    "", ""
                )?;
                write!(f, "\n{:indent$}END {options:?} JOIN", "")
            }
        }
    }
}

/// ALogicalPlan is a representation of LogicalPlan with Nodes which are allocated in an Arena
#[derive(Clone, Debug)]
pub enum ALogicalPlan {
    Selection {
        input: Node,
        predicate: Node,
    },
    DataFrameScan {
        #[cfg(feature = "built-in")]
        df: Option<Arc<DataFrame>>,
        schema: SchemaRef,
        // schema of the projected file
        output_schema: Option<SchemaRef>,
        projection: Option<Arc<Vec<String>>>,
        selection: Option<Node>,
    },
    Distinct {
        input: Node,
        options: DistinctOptions,
    },
    Projection {
        input: Node,
        expr: Vec<Node>,
        schema: SchemaRef,
    },
    Aggregate {
        input: Node,
        keys: Vec<Node>,
        aggs: Vec<Node>,
        schema: SchemaRef,
        apply: Option<Arc<dyn UserDefinedFunction>>,
        maintain_order: bool,
        // slice: Option<(i64, usize)>,
    },
    Join {
        input_left: Node,
        input_right: Node,
        schema: SchemaRef,
        left_on: Vec<Node>,
        right_on: Vec<Node>,
        options: JoinType,
    },

    /// A sink that indicates some internal logic error but not captured correctly.
    Nonsense(Node),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum ApplyOption {
    /// Collect groups to a list and apply the function over the groups.
    /// This can be important in aggregation context.
    // e.g. [g1, g1, g2] -> [[g1, g2], g2]
    ApplyGroups,
    // collect groups to a list and then apply
    // e.g. [g1, g1, g2] -> list([g1, g1, g2])
    ApplyList,
    // do not collect before apply
    // e.g. [g1, g1, g2] -> [g1, g1, g2]
    ApplyFlat,
}

impl Default for ALogicalPlan {
    fn default() -> Self {
        Self::Nonsense(Node::default())
    }
}

impl LogicalPlan {
    /// Returns the schema of the current logical plan.
    pub fn schema(&self) -> PolicyCarryingResult<SchemaRef> {
        match self {
            Self::Distinct { input, .. } | Self::Select { input, .. } => input.schema(),
            Self::Projection { schema, .. } => Ok(schema.clone()),
            Self::Aggregation { schema, .. } => Ok(schema.clone()),
            Self::Join { schema, .. } => Ok(schema.clone()),
            Self::StagedError { err, .. } => Err(err.clone()),
            Self::DataFrameScan { schema, .. } => Ok(schema.clone()),
        }
    }

    /// Moves `self` and converts it into a [`PlanBuilder`].
    pub fn into_builder(self) -> PlanBuilder {
        PlanBuilder::from(self)
    }

    /// Gets the inner policy.
    pub fn peek_policy(&self) -> Option<&Box<dyn Policy>> {
        match self {
            Self::Projection { policy, .. } => policy.as_ref(),
            _ => None,
        }
    }
}

impl ALogicalPlan {
    /// Returns the schema of the current arena-ed logical plan.
    pub fn schema(&self, lp_arena: &LogicalPlanArena) -> SchemaRef {
        match self {
            ALogicalPlan::DataFrameScan {
                schema,
                output_schema,
                ..
            } => output_schema.clone().unwrap_or(schema.clone()),
            ALogicalPlan::Distinct { input, .. } => lp_arena.get(*input).schema(lp_arena).clone(),
            ALogicalPlan::Aggregate { schema, .. } => schema.clone(),
            ALogicalPlan::Join { schema, .. } => schema.clone(),
            ALogicalPlan::Selection { input, .. } => lp_arena.get(*input).schema(lp_arena).clone(),
            ALogicalPlan::Projection { schema, .. } => schema.clone(),
            ALogicalPlan::Nonsense(_) => panic!("trying to access an invalid ALogicalPlan"),
        }
    }

    /// Gets the name.
    pub fn name(&self) -> &str {
        match self {
            ALogicalPlan::Aggregate { .. } => "Aggregate",
            ALogicalPlan::Distinct { .. } => "Distinct",
            ALogicalPlan::DataFrameScan { .. } => "Dataframe Scan",
            ALogicalPlan::Join { .. } => "Join",
            ALogicalPlan::Selection { .. } => "Selection",
            ALogicalPlan::Projection { .. } => "Projection",
            ALogicalPlan::Nonsense(_) => "Invalid!",
        }
    }
}

/// A wrapper that constructs a [`LogicalPlan`].
pub struct PlanBuilder {
    plan: LogicalPlan,
}

impl From<LogicalPlan> for PlanBuilder {
    fn from(plan: LogicalPlan) -> Self {
        Self { plan }
    }
}

impl From<DataFrame> for PlanBuilder {
    fn from(df: DataFrame) -> Self {
        let schema = df.schema();

        LogicalPlan::DataFrameScan {
            #[cfg(feature = "built-in")]
            df: Some(df.into()),
            schema,
            output_schema: None,
            projection: None,
            selection: None,
        }
        .into()
    }
}

impl PlanBuilder {
    /// Finishes the build and returns the inner struct.
    pub(crate) fn finish(self) -> LogicalPlan {
        self.plan
    }

    /// Performs aggregation and groupby.
    pub(crate) fn groupby<T: AsRef<[Expr]>>(
        self,
        keys: Vec<Expr>,
        expr: T,
        maintain_order: bool,
    ) -> Self {
        let schema = delayed_err!(self.plan.schema(), self.plan);
        // Group by what.
        let keys = delayed_err!(rewrite_projection(keys, schema.clone(), &[]), self.plan);
        let aggs = delayed_err!(
            rewrite_projection(expr.as_ref().to_vec(), schema.clone(), keys.as_ref()),
            self.plan
        );

        log::debug!("In groupby => {schema:?}, {keys:?}, {aggs:?}");

        let mut output_schema = delayed_err!(
            expressions_to_schema(keys.as_ref(), &schema, false),
            self.plan
        );

        let other_schema = delayed_err!(
            expressions_to_schema(aggs.as_ref(), &schema, true),
            self.plan
        );

        log::debug!("{output_schema:?} merge {other_schema:?}");
        output_schema.merge(other_schema);
        log::debug!("{output_schema:?}");

        // There contains some duplicate column names.
        if output_schema.fields_owned().len() < keys.len() + aggs.len() {
            let mut names = HashSet::new();
            for expr in aggs.iter().chain(keys.iter()) {
                let name = delayed_err!(expr_to_name(expr), self.plan);

                if !names.insert(name.clone()) {
                    return LogicalPlan::StagedError {
                        input: None,
                        err: PolicyCarryingError::DuplicateColumn(name),
                    }
                    .into();
                }
            }
        }

        LogicalPlan::Aggregation {
            input: Box::new(self.plan),
            schema: Arc::new(output_schema),
            keys: Arc::new(keys),
            aggs,
            apply: None,
            maintain_order,
            policy: None,
        }
        .into()
    }

    /// Joins the dataframe.
    pub(crate) fn join(
        self,
        other: LogicalPlan,
        left_on: Vec<Expr>,
        right_on: Vec<Expr>,
        join_type: JoinType,
    ) -> Self {
        // Check alias.
        for expr in left_on.iter().chain(right_on.iter()) {
            if matches!(expr, Expr::Alias { .. }) {
                return Self {
                    plan: LogicalPlan::StagedError {
                        input: Some(Box::new(self.plan)),
                        err: PolicyCarryingError::InvalidInput(
                            "cannot use `alias` in join predicates".into(),
                        ),
                    },
                };
            }
        }

        let schema_left = delayed_err!(self.plan.schema(), self.plan);
        let schema_right = delayed_err!(other.schema(), other);
        let output_schema = delayed_err!(
            schema_join(schema_left, schema_right, &left_on, &right_on, join_type),
            self.plan
        );

        Self {
            plan: LogicalPlan::Join {
                input_left: Box::new(self.plan),
                input_right: Box::new(other),
                schema: output_schema,
                left_on,
                right_on,
                options: join_type,
            },
        }
    }

    /// Performs projection.
    pub(crate) fn projection(self, expressions: Vec<Expr>) -> Self {
        let schema = delayed_err!(self.plan.schema(), self.plan);
        let expr = delayed_err!(
            rewrite_projection(
                expressions,
                schema.clone(),
                &[],
                // self.plan.peek_policy().map(|p| p.as_ref())
            ),
            self.plan
        );

        LogicalPlan::Projection {
            input: Box::new(self.plan),
            expression: expr,
            schema,
            policy: None,
        }
        .into()
    }

    /// Performs filtering.
    pub(crate) fn filter(self, expression: Expr) -> Self {
        // Check if the expression that should be normalized.
        let predicate = if expression
            .into_iter()
            .any(|e| matches!(e.deref(), Expr::Wildcard))
        {
            // "*" => expand to "filter(col)".
            let schema = delayed_err!(self.plan.schema(), self.plan);
            let expanded_columns = rewrite_projection(
                vec![expression],
                schema,
                &[],
                // self.plan.peek_policy().map(|policy| policy.as_ref()),
            );

            let mut expanded_columns = delayed_err!(expanded_columns, self.plan);
            if expanded_columns.is_empty() {
                return LogicalPlan::StagedError {
                    input: Some(Box::new(self.plan)),
                    err: PolicyCarryingError::ImpossibleOperation(
                        "trying to project on empty schema".into(),
                    ),
                }
                .into();
            } else if expanded_columns.len() == 1 {
                expanded_columns.pop().unwrap()
            } else {
                return LogicalPlan::StagedError {
                    input: Some(Box::new(self.plan)),
                    err: PolicyCarryingError::ImpossibleOperation(
                        "the predicate passed to 'LazyFrame.filter' expanded to multiple expressions".into(),
                    ),
                }
                .into();
            }
        } else {
            expression
        };

        Self {
            plan: LogicalPlan::Select {
                input: Box::new(self.plan),
                predicate,
                policy: None, // FIXME.
            },
        }
    }
}

/// Deals with the projection which may take the following forms:
///
/// * '*', exclude [column_1, ...];
/// * column_1, column_2, ...;
/// * column_1 * 2 (transformation: fn(FieldData) -> FieldData.
pub(crate) fn rewrite_projection(
    expressions: Vec<Expr>,
    schema: SchemaRef,
    keys: &[Expr],
) -> PolicyCarryingResult<Vec<Expr>> {
    let mut result = Vec::new();

    // Iterator over the expression list and try to
    for expression in expressions {
        log::debug!("rewriting the projection {expression:?}");

        match expression {
            Expr::Wildcard =>
            // We remove the wildcard projection "*" with the current schema because this selects 'all'.
            // Try to expand wildcard columns and then pushes them to the result set.
            {
                result.extend(
                    schema
                        .fields_owned()
                        .into_iter()
                        .map(|c| Expr::Column(c.name.clone())),
                )
            }

            Expr::Exclude(wildcard, columns) => {
                if matches!(*wildcard, Expr::Wildcard) {
                    result.extend(schema.fields_owned().into_iter().filter_map(|c| {
                        if columns.contains(&c.name) {
                            Some(Expr::Column(c.name.clone()))
                        } else {
                            None
                        }
                    }));
                }
            }

            Expr::Agg(agg) => {
                let mut new_expr =
                    rewrite_projection(vec![agg.as_expr().clone()], schema.clone(), keys)?
                        .into_iter()
                        .map(|expr| match agg {
                            Aggregation::Max(_) => Expr::Agg(Aggregation::Max(Box::new(expr))),
                            Aggregation::Min(_) => Expr::Agg(Aggregation::Min(Box::new(expr))),
                            Aggregation::Sum(_) => Expr::Agg(Aggregation::Sum(Box::new(expr))),
                            Aggregation::Mean(_) => Expr::Agg(Aggregation::Mean(Box::new(expr))),
                        })
                        .collect::<Vec<_>>();
                result.append(&mut new_expr);
            }

            _ => result.push(expression),
        }
    }

    // Check if all the column names are unique.
    let mut set = HashSet::new();
    for name in result.iter().filter_map(|expr| match expr {
        Expr::Column(ref name) => Some(name),
        _ => None,
    }) {
        // TODO: Add qualifier for ambiguous column names. E.g., A.c, B.c => full quantifier!
        pcd_ensures!(set.insert(name.clone()), DuplicateColumn:       "found duplicate column name {name}");
    }

    Ok(result)
}

/// Performs the optimization on the [`LogicalPlan`], if needed.
#[cfg_attr(not(feature = "optimize"), allow(unused_variables))]
pub(crate) fn optimize(
    logical_plan: LogicalPlan,
    opt_flag: OptFlag,
    lp_arena: &mut LogicalPlanArena,
    expr_arena: &mut ExprArena,
    nodes: &mut Vec<Node>,
) -> PolicyCarryingResult<Node> {
    #[cfg(feature = "optimize")]
    {
        // If there is nothing that we perform optimization on, we directly returns the node identifier.
        if opt_flag.contains(OptFlag::NONE) {
            lp_to_alp(logical_plan, expr_arena, lp_arena)
        } else {
            todo!()
        }
    }

    #[cfg(not(feature = "optimize"))]
    {
        // Since we do no implement optimization at this timepoint, we just call `lp_to_alp`.
        lp_to_alp(logical_plan, expr_arena, lp_arena)
    }
}

/// Converts the aggregation expression into its arena-ed version [`AAggExpr`].
pub(crate) fn agg_to_aagg(
    aggregation: Aggregation,
    expr_arena: &mut ExprArena,
) -> PolicyCarryingResult<AAggExpr> {
    let expr = match aggregation {
        Aggregation::Min(expr) => AAggExpr::Min {
            input: expr_to_aexpr(*expr, expr_arena)?,
            // FIXME: Propagate?
            propagate_nans: true,
        },
        Aggregation::Max(expr) => AAggExpr::Max {
            input: expr_to_aexpr(*expr, expr_arena)?,
            // FIXME: Propagate?
            propagate_nans: true,
        },
        Aggregation::Sum(expr) => AAggExpr::Sum(expr_to_aexpr(*expr, expr_arena)?),
        Aggregation::Mean(expr) => AAggExpr::Mean(expr_to_aexpr(*expr, expr_arena)?),
    };

    Ok(expr)
}

/// Taking as input the [`Expr`] and two arenas for storing the intermediate results, this function
/// converts the [`AExpr`] to its corresponding [`Node`] in the arena for physical plan generation.
pub(crate) fn expr_to_aexpr(expr: Expr, expr_arena: &mut ExprArena) -> PolicyCarryingResult<Node> {
    let aexpr = match expr {
        Expr::Agg(aggregation) => AExpr::Agg(agg_to_aagg(aggregation, expr_arena)?),
        Expr::Column(name) => AExpr::Column(name),
        Expr::Count => AExpr::Count,
        Expr::Alias { expr, name } => {
            let expr = expr_to_aexpr(*expr, expr_arena)?;
            AExpr::Alias(expr, name)
        }
        Expr::Wildcard => AExpr::Wildcard,
        Expr::Exclude(_, _) => panic!("exclude should be expanded at the higher level."),
        Expr::Filter { input, filter } => {
            let input = expr_to_aexpr(*input, expr_arena)?;
            let predicate = expr_to_aexpr(*filter, expr_arena)?;
            AExpr::Filter {
                input,
                by: predicate,
            }
        }
        Expr::BinaryOp { left, op, right } => {
            let left: Node = expr_to_aexpr(*left, expr_arena)?;
            let right = expr_to_aexpr(*right, expr_arena)?;
            AExpr::BinaryOp { left, op, right }
        }
        Expr::Literal(val) => AExpr::Literal(val),
    };

    Ok(expr_arena.add(aexpr))
}

/// Taking as input the [`LogicalPlan`] and two arenas for storing the intermediate results, this function
/// converts the [`LogicalPlan`] to its corresponding [`Node`] in the arena for physical plan generation.
/// Note that the input logical plan may be optimized to get a better performance.
pub(crate) fn lp_to_alp(
    logical_plan: LogicalPlan,
    expr_arena: &mut ExprArena,
    lp_arena: &mut LogicalPlanArena,
) -> PolicyCarryingResult<Node> {
    let alp = match logical_plan {
        LogicalPlan::Select {
            input, predicate, ..
        } => {
            let input = lp_to_alp(*input, expr_arena, lp_arena)?;
            let predicate = expr_to_aexpr(predicate, expr_arena)?;
            ALogicalPlan::Selection { input, predicate }
        }
        LogicalPlan::Distinct { input, options } => {
            let input = lp_to_alp(*input, expr_arena, lp_arena)?;
            ALogicalPlan::Distinct { input, options }
        }
        LogicalPlan::Projection {
            input,
            expression,
            schema,
            ..
        } => {
            let input = lp_to_alp(*input, expr_arena, lp_arena)?;
            let expression = expression
                .into_iter()
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .collect::<PolicyCarryingResult<Vec<_>>>()?;
            ALogicalPlan::Projection {
                input,
                expr: expression,
                schema,
            }
        }

        LogicalPlan::DataFrameScan {
            #[cfg(feature = "built-in")]
            df,
            projection,
            selection,
            schema,
            output_schema,
        } => ALogicalPlan::DataFrameScan {
            #[cfg(feature = "built-in")]
            df,
            schema,
            output_schema,
            projection,
            selection: selection
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .transpose()?,
        },
        LogicalPlan::Aggregation {
            input,
            schema,
            keys,
            aggs,
            apply,
            maintain_order,
            ..
        } => {
            let input = lp_to_alp(*input, expr_arena, lp_arena)?;

            // Try unwrap the reference counter and clone it if necessary.
            let keys = Arc::try_unwrap(keys)
                .unwrap_or_else(|keys| keys.deref().clone())
                .into_iter()
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .collect::<PolicyCarryingResult<Vec<_>>>()?;
            let aggs = aggs
                .into_iter()
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .collect::<PolicyCarryingResult<Vec<_>>>()?;

            ALogicalPlan::Aggregate {
                input,
                keys,
                aggs,
                schema,
                apply,
                maintain_order,
            }
        }
        LogicalPlan::Join {
            input_left,
            input_right,
            schema,
            left_on,
            right_on,
            options,
        } => {
            let input_left = lp_to_alp(*input_left, expr_arena, lp_arena)?;
            let input_right = lp_to_alp(*input_right, expr_arena, lp_arena)?;
            let left_on = left_on
                .into_iter()
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .collect::<PolicyCarryingResult<Vec<_>>>()?;
            let right_on = right_on
                .into_iter()
                .map(|expr| expr_to_aexpr(expr, expr_arena))
                .collect::<PolicyCarryingResult<Vec<_>>>()?;

            ALogicalPlan::Join {
                input_left,
                input_right,
                schema,
                left_on,
                right_on,
                options,
            }
        }
        LogicalPlan::StagedError { input, err } => {
            log::error!(
                "Error occurrred while evaluating logical plan. The last visited plan was {input:?}"
            );
            return Err(err);
        }
    };

    Ok(lp_arena.add(alp))
}

/// This function converts the logical plan [`LogicalPlan`] into a [`PhysicalPlan`] and also
/// applies some optimizations thereon for best performance. Meanwhile, this function will
/// analyze if the query plan would have any change that it will break the given privacy
/// policy or apply some necessary privacy schemes on the data (hints the executor).
pub(crate) fn make_physical_plan(
    lp: LogicalPlan,
    opt_flag: OptFlag,
    executor_ref_id: Option<ExecutorRefId>,
) -> PolicyCarryingResult<PhysicalPlan> {
    // Create two arenas for expressions and logical plans (for their optimizations).
    let mut expr_arena = ExprArena::with_capacity(EXPR_ARENA_SIZE);
    let mut lp_arena = LogicalPlanArena::with_capacity(LP_ARENA_SIZE);
    // Store nodes.
    let mut nodes = Vec::new();

    let root = optimize(lp, opt_flag, &mut lp_arena, &mut expr_arena, &mut nodes)?;

    #[cfg(feature = "builtin")]
    let executor = do_make_physical_plan(root, &mut lp_arena, &mut expr_arena)?;

    #[cfg(not(feature = "builtin"))]
    let executor = do_make_physical_plan(root, &mut lp_arena, &mut expr_arena, executor_ref_id)?;

    Ok((ExecutionState::with_executors(executor_ref_id), executor))
}

/// A recursive function that handles the conversion from [`ALogicalPlan`] to the [`OpaquePtr`]
/// which points to a valid [``].
fn do_make_physical_plan(
    root: Node,
    lp_arena: &mut LogicalPlanArena,
    expr_arena: &mut ExprArena,
    executor_ref_id: Option<ExecutorRefId>,
) -> PolicyCarryingResult<ExecutorHandle> {
    let node = lp_arena.take(root);

    log::debug!("visiting node {node:?}");

    match node {
        ALogicalPlan::Selection { input, predicate } => {
            let input = do_make_physical_plan(input, lp_arena, expr_arena, executor_ref_id)?;
            let state = ExecutionState::with_executors(executor_ref_id);
            let predicate = make_physical_expr(predicate, expr_arena, None, &state, false)?;

            log::debug!("creating executor: Filter");

            match input {
                #[cfg(feature = "built-in")]
                ExecutorHandle::Owned(input) => Ok(ExecutorHandle::Owned(Box::new(
                    crate::executor::filter::FilterExec::new(predicate, input),
                ))),
                #[cfg(not(feature = "built-in"))]
                ExecutorHandle::Owned(executor) => Err(PolicyCarryingError::OperationNotSupported(
                    "no built-in executors available".into(),
                )),

                ExecutorHandle::Ptr(input) => create_executor(
                    executor_ref_id,
                    args! {
                        "executor_type": serde_json::to_string(&ExecutorType::Filter).unwrap(),
                        "input": input as usize,
                        "predicate": serde_json::to_string(&predicate).unwrap(),
                    },
                ),
            }
        }
        ALogicalPlan::DataFrameScan {
            #[cfg(feature = "built-in")]
            df,
            schema,
            projection,
            selection,
            ..
        } => {
            let state = ExecutionState::with_executors(executor_ref_id);
            let selection = match selection {
                Some(node) => {
                    match make_physical_expr(node, expr_arena, Some(schema.clone()), &state, false)
                    {
                        Ok(selection) => Some(selection),
                        Err(_) => None,
                    }
                }
                None => None,
            };

            log::debug!("creating executor: DataFrameScan");

            if executor_ref_id.is_none() {
                #[cfg(feature = "built-in")]
                return Ok(ExecutorHandle::Owned(Box::new(
                    crate::executor::scan::DataFrameExec::new(df, selection, projection, false),
                )));

                #[cfg(not(feature = "built-in"))]
                return Err(PolicyCarryingError::OperationNotSupported(
                    "no built-in executors available",
                ));
            } else {
                create_executor(
                    executor_ref_id,
                    args! {
                        "executor_type": serde_json::to_string(&ExecutorType::DataframeScan).unwrap(),
                        "schema": serde_json::to_string(&schema).unwrap(),
                        "projection": projection.map(|proj| proj.deref().clone()),
                        "predicate_has_windows": false,
                        "selection": selection.map(|sel| serde_json::to_string(&sel).unwrap()),
                    },
                )
            }
        }
        ALogicalPlan::Distinct { input, options } => {
            let input = do_make_physical_plan(input, lp_arena, expr_arena, executor_ref_id)?;

            match input {
                #[cfg(feature = "built-in")]
                ExecutorHandle::Owned(input) => Ok(ExecutorHandle::Owned(Box::new(
                    crate::executor::distinct::DistinctExec::new(input, options),
                ))),
                #[cfg(not(feature = "built-in"))]
                ExecutorHandle::Owned(executor) => Err(PolicyCarryingError::OperationNotSupported(
                    "no built-in executors available".into(),
                )),
                ExecutorHandle::Ptr(input) => create_executor(
                    executor_ref_id,
                    args! {
                        "executor_type": serde_json::to_string(&ExecutorType::Projection).unwrap(),
                        "input": input as usize,
                        "options": serde_json::to_string(&options).unwrap(),
                    },
                ),
            }
        }
        ALogicalPlan::Projection { input, expr, .. } => {
            let schema = lp_arena.get(input).schema(lp_arena);
            let input = do_make_physical_plan(input, lp_arena, expr_arena, executor_ref_id)?;
            let state = ExecutionState::with_executors(executor_ref_id);
            let expr = expr
                .into_iter()
                .map(|expr| {
                    make_physical_expr(expr, expr_arena, Some(schema.clone()), &state, false)
                })
                .collect::<PolicyCarryingResult<Vec<_>>>()?;

            log::debug!("creating executor: Projection");

            match input {
                #[cfg(feature = "built-in")]
                ExecutorHandle::Owned(input) => Ok(ExecutorHandle::Owned(Box::new(
                    crate::executor::projection::ProjectionExec::new(input, expr, schema),
                ))),
                #[cfg(not(feature = "built-in"))]
                ExecutorHandle::Owned(executor) => Err(PolicyCarryingError::OperationNotSupported(
                    "no built-in executors available".into(),
                )),
                ExecutorHandle::Ptr(input) => create_executor(
                    executor_ref_id,
                    args! {
                        "executor_type": serde_json::to_string(&ExecutorType::Projection).unwrap(),
                        "input": input as usize,
                        "expr": serde_json::to_string(&expr).unwrap(),
                        "input_schema": serde_json::to_string(&schema).unwrap(),
                    },
                ),
            }
        }
        ALogicalPlan::Aggregate {
            input,
            keys,
            aggs,
            schema,
            apply,
            maintain_order,
        } => {
            if let Some(ref apply) = apply {
                unimplemented!("`groupby` with {apply:?} is not supported at this time.");
            }

            let input_schema = lp_arena.get(input).schema(lp_arena);
            let state = ExecutionState::with_executors(executor_ref_id);
            let phys_aggs = aggs
                .iter()
                .map(|expr| {
                    make_physical_expr(*expr, expr_arena, Some(input_schema.clone()), &state, true)
                })
                .collect::<PolicyCarryingResult<Vec<_>>>()?;
            // groupby.
            let phys_keys = keys
                .iter()
                .map(|expr| {
                    make_physical_expr(*expr, expr_arena, Some(input_schema.clone()), &state, true)
                })
                .collect::<PolicyCarryingResult<Vec<_>>>()?;

            log::debug!("{schema:?}, {phys_aggs:#?}, {phys_keys:?}");

            if partitionable(keys.as_ref(), aggs.as_ref(), expr_arena) {
                log::debug!("partitionable!");

                let input = do_make_physical_plan(input, lp_arena, expr_arena, executor_ref_id)?;
                let keys = keys
                    .iter()
                    .map(|node| aexpr_to_expr(*node, expr_arena))
                    .collect::<Vec<_>>();
                let aggs = aggs
                    .iter()
                    .map(|node| aexpr_to_expr(*node, expr_arena))
                    .collect::<Vec<_>>();

                log::debug!("creating executor: PartitionGroupBy");

                match input {
                    #[cfg(feature = "built-in")]
                    ExecutorHandle::Owned(input) => Ok(ExecutorHandle::Owned(Box::new(
                        crate::executor::groupby_partitioned::PartitionGroupByExec {
                            input,
                            phys_keys,
                            phys_aggs,
                            maintain_order,
                            slice: None,
                            input_schema,
                            output_schema: schema,
                            keys,
                            aggs,
                            from_partitioned_ds: false,
                        },
                    ))),
                    #[cfg(not(feature = "built-in"))]
                    ExecutorHandle::Owned(executor) => {
                        Err(PolicyCarryingError::OperationNotSupported(
                            "no built-in executors available".into(),
                        ))
                    }

                    ExecutorHandle::Ptr(input) => create_executor(
                        executor_ref_id,
                        args! {
                            "executor_type": serde_json::to_string(&ExecutorType::PartitionGroupBy).unwrap(),
                            "input": input as usize,
                            "phys_keys": serde_json::to_string(&phys_keys).unwrap(),
                            "phys_aggs": serde_json::to_string(&phys_aggs).unwrap(),
                            "maintain_order": maintain_order,
                            "slice": serde_json::to_string(&None::<(i64, usize)>).unwrap(),
                            "input_schema": serde_json::to_string(&input_schema).unwrap(),
                            "output_schema": serde_json::to_string(&schema).unwrap(),
                            // "from_partitioned_ds": false,
                            "keys": serde_json::to_string(&keys).unwrap(),
                            "aggs": serde_json::to_string(&aggs).unwrap(),
                        },
                    ),
                }
            } else {
                log::debug!("not partitionable!");
                todo!()
            }
        }

        ALogicalPlan::Join {
            input_left,
            input_right,
            schema,
            left_on,
            right_on,
            options,
        } => {
            // FIXME: Add multiple executor reference IDs?
            let input_left =
                do_make_physical_plan(input_left, lp_arena, expr_arena, executor_ref_id)?;
            let input_right =
                do_make_physical_plan(input_right, lp_arena, expr_arena, executor_ref_id)?;

            pcd_ensures!(
                input_left.is_same_type(&input_right),
                ImpossibleOperation: "the left and right handle have different types {:?} vs {:?} which is not allowed",
                input_left,
                input_right,
            );

            let left_on = left_on
                .into_iter()
                .map(|expr| {
                    make_physical_expr(
                        expr,
                        expr_arena,
                        Some(schema.clone()),
                        &Default::default(),
                        false,
                    )
                })
                .collect::<PolicyCarryingResult<Vec<_>>>()?;
            let right_on = right_on
                .into_iter()
                .map(|expr| {
                    make_physical_expr(
                        expr,
                        expr_arena,
                        Some(schema.clone()),
                        &Default::default(),
                        false,
                    )
                })
                .collect::<PolicyCarryingResult<Vec<_>>>()?;

            match (input_left, input_right) {
                (ExecutorHandle::Ptr(input_left), ExecutorHandle::Ptr(input_right)) => {
                    create_executor(
                        executor_ref_id,
                        args! {
                            "input_left": input_left as usize,
                            "input_right": input_right as usize,
                        },
                    )
                }
                (ExecutorHandle::Owned(input_left), ExecutorHandle::Owned(input_right)) => Ok(
                    ExecutorHandle::Owned(Box::new(crate::executor::join::JoinExec::new(
                        input_left,
                        input_right,
                        left_on,
                        right_on,
                        options,
                    ))),
                ),

                _ => panic!("logic error"),
            }
        }

        ALogicalPlan::Nonsense(_) => Err(PolicyCarryingError::InvalidInput(
            "nonsence logical plan encountered".into(),
        )),
    }
}

/// Converts the arena-ed expression into its concrete type from the [`ExprArena`].
pub(crate) fn aexpr_to_expr(aexpr: Node, expr_arena: &ExprArena) -> Expr {
    let aexpr = expr_arena.get(aexpr).clone();

    match aexpr {
        AExpr::Alias(input, name) => Expr::Alias {
            expr: Box::new(aexpr_to_expr(input, expr_arena)),
            name,
        },
        AExpr::Count => Expr::Count,
        AExpr::Column(name) => Expr::Column(name),
        AExpr::Literal(val) => Expr::Literal(val),
        AExpr::BinaryOp { left, op, right } => Expr::BinaryOp {
            left: Box::new(aexpr_to_expr(left, expr_arena)),
            op,
            right: Box::new(aexpr_to_expr(right, expr_arena)),
        },
        AExpr::Filter { input, by } => Expr::Filter {
            input: Box::new(aexpr_to_expr(input, expr_arena)),
            filter: Box::new(aexpr_to_expr(by, expr_arena)),
        },
        AExpr::Agg(agg) => match agg {
            AAggExpr::Max { input, .. } => {
                Expr::Agg(Aggregation::Max(Box::new(aexpr_to_expr(input, expr_arena))))
            }
            AAggExpr::Min { input, .. } => {
                Expr::Agg(Aggregation::Min(Box::new(aexpr_to_expr(input, expr_arena))))
            }
            AAggExpr::Sum(input) => {
                Expr::Agg(Aggregation::Sum(Box::new(aexpr_to_expr(input, expr_arena))))
            }
            AAggExpr::Mean(input) => Expr::Agg(Aggregation::Mean(Box::new(aexpr_to_expr(
                input, expr_arena,
            )))),
        },
        AExpr::Wildcard => Expr::Wildcard,
    }
}

/// This function checks:
///      1. complex expressions in the groupby itself are also not partitionable
///          in this case anything more than `col("foo")`.
///      2. (not checked) a custom function cannot be partitioned.
///      3. we don't bother with more than 2 keys, as the cardinality likely explodes
///         by the combinations.
///
/// Taken from polars.
pub(crate) fn partitionable(keys: &[Node], aggs: &[Node], expr_arena: &ExprArena) -> bool {
    let mut partitionable = true;

    if !keys.is_empty() && keys.len() < 3 {
        // complex expressions in the groupby itself are also not partitionable
        // in this case anything more than col("foo")
        for key in keys {
            if expr_arena.iter(*key).count() > 1 {
                partitionable = false;
                break;
            }
        }

        if partitionable {
            for agg in aggs {
                let aexpr = expr_arena.get(*agg);

                // COUNT(*) is definitely partitionable, but it only has depth 1.
                // So we skip this case.
                if matches!(aexpr, AExpr::Count) {
                    continue;
                }

                let depth = (expr_arena).iter(*agg).count();

                if depth == 1 {
                    partitionable = false;
                    break;
                }

                // it should end with an aggregation
                if let AExpr::Alias(_, _) = aexpr {
                    // col().agg().alias() is allowed: count of 3
                    // col().alias() is not allowed: count of 2
                    // count().alias() is allowed: count of 2
                    if depth <= 2 {
                        partitionable = false;
                        break;
                    }
                }

                let has_aggregation = |node: Node| {
                    expr_arena
                        .iter(node)
                        .any(|(_, ae)| matches!(ae, AExpr::Agg(_) | AExpr::Count))
                };

                // check if the aggregation type is partitionable
                // only simple aggregation like col().sum
                // that can be divided in to the aggregation of their partitions are allowed
                if !((expr_arena).iter(*agg).all(|(_, ae)| match ae {
                    AExpr::Agg(agg_e) => matches!(
                        agg_e,
                        AAggExpr::Min { .. } | AAggExpr::Max { .. } | AAggExpr::Sum(_)
                    ),
                    AExpr::Count | AExpr::Column(_) | AExpr::Literal(_) | AExpr::Alias(_, _) => {
                        true
                    }
                    AExpr::BinaryOp { left, right, .. } => {
                        !has_aggregation(*left) && !has_aggregation(*right)
                    }
                    _ => false,
                })) && matches!(aexpr, AExpr::Alias(_, _) | AExpr::Agg(_))
                {
                    partitionable = false;
                    break;
                }
            }
        }
    } else {
        partitionable = keys.is_empty();
    }
    partitionable
}

/// Given two schemas of the input dataframes, outputs the joined schema.
fn schema_join(
    schema_left: SchemaRef,
    schema_right: SchemaRef,
    left_on: &[Expr],
    right_on: &[Expr],
    _join_type: JoinType,
) -> PolicyCarryingResult<SchemaRef> {
    let mut sb = SchemaBuilder::from(schema_left.clone());
    let mut left_names = HashSet::with_capacity(schema_left.fields().len());

    for field in schema_left.fields() {
        left_names.insert(field.name.clone());
    }

    let mut right_names = HashSet::with_capacity(right_on.len());
    for expr in left_on {
        let field = expr_to_field(expr, &schema_left, false)?;
        sb = sb.add_field(field);
    }
    for expr in right_on {
        let field = expr_to_field(expr, &schema_left, false)?;
        right_names.insert(field.name.clone());
    }

    for right_field in schema_right.fields() {
        let right_name = right_field.name.as_str();
        // We found some field whose name does not appear in the `right_on` but appear in the left schema.
        // In this case, we need to disambiguify the column names of the left and right schemas.
        //
        // For example:
        //
        // D B C  JOIN  B C D   ON      lhs.B = rhs.B && lhs.C = rhs.C but lhs.D != rhs.D
        if !right_names.contains(right_name) {
            if left_names.contains(right_name) {
                let right_name = format!("{right_name}_RHS");
                sb = sb.add_field_raw(&right_name, right_field.data_type, right_field.nullable);
            } else {
                // Push.
                sb = sb.add_field(right_field.clone());
            }
        }
    }

    let schema = sb.finish();
    log::debug!("The joined schema would be {schema:?}");

    Ok(schema)
}
