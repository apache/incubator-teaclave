use std::fmt::{Debug, Formatter};

use policy_carrying_data::field::FieldDataRef;
use policy_core::error::PolicyCarryingResult;
use serde::{Deserialize, Serialize};

/// A user defiend function that can be applied on a mutable array of [`FieldDataRef`].
#[typetag::serde(tag = "udf")]
pub trait UserDefinedFunction: Send + Sync {
    fn call(&self, input: &mut [FieldDataRef]) -> PolicyCarryingResult<Option<FieldDataRef>>;
}

impl Debug for dyn UserDefinedFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "UDF")
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UdfWrapper {
    /// Call the udf later to allow de-/serialization.
    pub udf: String,
}

#[typetag::serde]
impl UserDefinedFunction for UdfWrapper {
    fn call(&self, _input: &mut [FieldDataRef]) -> PolicyCarryingResult<Option<FieldDataRef>> {
        todo!()
    }
}
