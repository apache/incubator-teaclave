use policy_core::{error::PolicyCarryingError, types::FunctionArguments};
use policy_utils::move_box_ptr;

use crate::plan::physical_expr::PhysicalExprRef;

use super::Executor;

#[derive(Clone)]
pub struct FilterExec {
    pub predicate: PhysicalExprRef,
    pub input: Executor,
}

impl FilterExec {
    pub fn new(predicate: PhysicalExprRef, input: Executor) -> Self {
        Self { predicate, input }
    }
}

impl TryFrom<FunctionArguments> for FilterExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let predicate = args.get_and_apply("predicate", |predicate: String| {
            serde_json::from_str(&predicate)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let input = args.get_and_apply("input", |ptr: usize| move_box_ptr(ptr as *mut Executor))?;

        Ok(Self::new(predicate, input))
    }
}
