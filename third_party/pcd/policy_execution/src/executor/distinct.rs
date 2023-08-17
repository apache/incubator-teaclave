use std::fmt::{Debug, Formatter};

use policy_carrying_data::FunctionArguments;
use policy_core::{error::PolicyCarryingError, expr::DistinctOptions};
use policy_utils::move_box_ptr;

use super::Executor;

#[derive(Clone)]
pub struct DistinctExec {
    pub input: Executor,
    pub options: DistinctOptions,
}

impl Debug for DistinctExec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DistinctExec")
    }
}

impl DistinctExec {
    #[inline]
    pub fn new(input: Executor, options: DistinctOptions) -> Self {
        Self { input, options }
    }
}

impl TryFrom<FunctionArguments> for DistinctExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        // Migrate to RPC!
        let input = args.get_and_apply("input", |input: usize| move_box_ptr(input as _))?;
        let options = args.get_and_apply("options", |options: String| {
            serde_json::from_str(&options)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;

        Ok(Self::new(input, options))
    }
}
