use std::sync::Arc;

use policy_carrying_data::{field::FieldDataRef, schema::SchemaRef, DataFrame};
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::Expr,
    get_lock, pcd_ensures,
    types::FunctionArguments,
};
use policy_utils::move_box_ptr;

use crate::{plan::physical_expr::PhysicalExprRef, udf::UserDefinedFunction};

use super::{ExecutionState, Executor};

#[derive(Clone)]
pub struct PartitionGroupByExec {
    pub input: Executor,
    pub phys_keys: Vec<PhysicalExprRef>,
    pub phys_aggs: Vec<PhysicalExprRef>,
    pub maintain_order: bool,
    pub slice: Option<(i64, usize)>,
    pub input_schema: SchemaRef,
    pub output_schema: SchemaRef,
    pub from_partitioned_ds: bool,
    pub keys: Vec<Expr>,
    pub aggs: Vec<Expr>,
}

impl TryFrom<FunctionArguments> for PartitionGroupByExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let input =
            args.get_and_apply("input", |input: usize| move_box_ptr(input as *mut Executor))?;
        let phys_keys = args.get_and_apply("phys_keys", |phys_keys: String| {
            serde_json::from_str(&phys_keys)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let phys_aggs = args.get_and_apply("phys_aggs", |phys_aggs: String| {
            serde_json::from_str(&phys_aggs)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let maintain_order =
            args.get_and_apply("maintain_order", |maintain_order: bool| maintain_order)?;
        let slice = args.get_and_apply("slice", |slice: String| {
            serde_json::from_str(&slice)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let input_schema = args.get_and_apply("input_schema", |input_schema: String| {
            serde_json::from_str(&input_schema)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let output_schema = args.get_and_apply("output_schema", |output_schema: String| {
            serde_json::from_str(&output_schema)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let keys = args.get_and_apply("keys", |keys: String| {
            serde_json::from_str(&keys)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let aggs = args.get_and_apply("aggs", |aggs: String| {
            serde_json::from_str(&aggs)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;

        Ok(Self {
            input,
            phys_keys,
            phys_aggs,
            maintain_order,
            slice,
            input_schema,
            output_schema,
            from_partitioned_ds: false,
            keys,
            aggs,
        })
    }
}

impl PartitionGroupByExec {
    /// Computes the aggregation keys from the physical expressions.
    pub fn keys(
        &self,
        df: &DataFrame,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<Vec<FieldDataRef>> {
        self.phys_keys
            .iter()
            .map(|key| key.evaluate(df, state))
            .collect()
    }

    /// Tries to partition the original dataframe into multiple ones.
    pub fn partition_dataframe(
        &self,
        df: &mut DataFrame,
        maintain_order: bool,
        state: &ExecutionState,
    ) -> PolicyCarryingResult<Vec<DataFrame>> {
        let keys = self.keys(df, state)?;
        let gb = df.groupby_with_keys(keys, maintain_order)?;

        println!("groupby helper => {gb:?}");

        todo!()
    }

    /// The internal implementation of the execution on which the privacy scheme can be further applied.
    pub fn execute_impl(
        &self,
        state: &ExecutionState,
        original_df: DataFrame,
    ) -> PolicyCarryingResult<DataFrame> {
        let keys = self.keys(&original_df, state)?;

        log::debug!("get keys => {keys:?}");

        groupby_helper(
            original_df,
            keys,
            &self.phys_aggs,
            None,
            state,
            self.maintain_order,
            None,
        )
    }
}

/// The default hash aggregation algorithm.
/// BUG.
pub(crate) fn groupby_helper(
    df: DataFrame,
    keys: Vec<FieldDataRef>,
    aggs: &[PhysicalExprRef],
    apply: Option<Arc<dyn UserDefinedFunction>>,
    state: &ExecutionState,
    maintain_order: bool,
    slice: Option<(usize, usize)>,
) -> PolicyCarryingResult<DataFrame> {
    let gb = df.groupby_with_keys(keys, maintain_order)?;

    pcd_ensures!(apply.is_none(), OperationNotSupported: "cannot use apply");

    get_lock!(state.expr_cache, lock).clear();
    let mut columns = gb.keys_sliced(slice);
    let aggs = aggs
        .into_iter()
        .map(|expr| {
            let agg = expr.evaluate_groups(&df, &gb.proxy, state)?.finalize();
            Ok(agg)
        })
        .collect::<PolicyCarryingResult<Vec<_>>>()?;
    get_lock!(state.expr_cache, lock).clear();

    columns.extend_from_slice(aggs.as_slice());
    Ok(DataFrame::new_with_cols(columns))
}
