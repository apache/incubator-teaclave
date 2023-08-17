//! The join executor

use policy_carrying_data::{FunctionArguments, JoinType};
use policy_core::error::PolicyCarryingError;
use policy_utils::move_box_ptr;

use crate::plan::physical_expr::PhysicalExprRef;

use super::Executor;

#[derive(Clone)]
pub struct JoinExec {
    /// The left dataframe executor.
    pub input_left: Executor,
    /// The right dataframe executor.
    pub input_right: Executor,
    /// The left on predicate.
    pub left_on: Vec<PhysicalExprRef>,
    /// The right on predicate.
    pub right_on: Vec<PhysicalExprRef>,
    /// The join type.
    pub join_type: JoinType,
}

impl JoinExec {
    pub fn new(
        input_left: Executor,
        input_right: Executor,
        left_on: Vec<PhysicalExprRef>,
        right_on: Vec<PhysicalExprRef>,
        join_type: JoinType,
    ) -> Self {
        Self {
            input_left,
            input_right,
            left_on,
            right_on,
            join_type,
        }
    }
}

impl TryFrom<FunctionArguments> for JoinExec {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let input_left = args.get_and_apply("input_left", |input_left: usize| {
            move_box_ptr(input_left as _)
        })?;
        let input_right = args.get_and_apply("input_right", |input_right: usize| {
            move_box_ptr(input_right as _)
        })?;
        let left_on = args.get_and_apply("left_on", |left_on: String| {
            serde_json::from_str(&left_on)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let right_on = args.get_and_apply("right_on", |right_on: String| {
            serde_json::from_str(&right_on)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;
        let join_type = args.get_and_apply("join_type", |join_type: String| {
            serde_json::from_str::<JoinType>(&join_type)
                .map_err(|e| PolicyCarryingError::SerializeError(e.to_string()))
        })??;

        Ok(Self::new(
            input_left,
            input_right,
            left_on,
            right_on,
            join_type,
        ))
    }
}
