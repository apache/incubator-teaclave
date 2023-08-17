use policy_core::{error::PolicyCarryingError, expr::GroupByMethod, types::FunctionArguments};
use policy_utils::move_box_ptr;

use super::Executor;

#[derive(Clone)]
pub struct ApplyExec {
    pub input: Executor,
    /// The type of this apply executor.
    pub method: GroupByMethod,
}

impl ApplyExec {
    pub fn new(input: Executor, method: GroupByMethod) -> Self {
        Self { input, method }
    }
}

impl TryFrom<FunctionArguments> for ApplyExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let input =
            args.get_and_apply("input", |input: usize| move_box_ptr(input as *mut Executor))?;
        let method = args.get_and_apply("method", |method: GroupByMethod| method)?;

        Ok(Self::new(input, method))
    }
}
