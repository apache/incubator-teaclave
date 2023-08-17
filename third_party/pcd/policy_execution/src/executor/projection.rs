use policy_carrying_data::schema::{Schema, SchemaRef};
use policy_core::{error::PolicyCarryingError, types::FunctionArguments};
use policy_utils::move_box_ptr;

use crate::plan::physical_expr::PhysicalExprRef;

use super::Executor;

/// Implementes the physical executor for projection.
#[derive(Clone)]
pub struct ProjectionExec {
    pub input: Executor,
    pub expr: Vec<PhysicalExprRef>,
    pub input_schema: SchemaRef,
}

impl ProjectionExec {
    pub fn new(input: Executor, expr: Vec<PhysicalExprRef>, input_schema: SchemaRef) -> Self {
        Self {
            input,
            expr,
            input_schema,
        }
    }
}

impl TryFrom<FunctionArguments> for ProjectionExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let input = args.get_and_apply("input", |ptr: usize| move_box_ptr(ptr as *mut Executor))?;
        let expr = args.get_and_apply("expr", move |expr: String| {
            serde_json::from_str(&expr)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;

        let input_schema = args.get_and_apply("input_schema", |input_schema: String| {
            serde_json::from_str::<Schema>(&input_schema)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;

        Ok(ProjectionExec::new(input, expr, input_schema.into()))
    }
}
