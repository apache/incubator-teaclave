use std::{
    any::Any,
    cmp::Ordering,
    ffi::c_void,
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
};

use num_enum::{FromPrimitive, IntoPrimitive};
use num_traits::Zero;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::error::{PolicyCarryingError, PolicyCarryingResult};

pub type OpaquePtr = *mut c_void;

/// A wrapper ID for bookkeeping the executor sets.
#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize,
)]
#[repr(C)]
pub struct ExecutorRefId(pub usize);

impl Display for ExecutorRefId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The set of datatypes that are supported. Typically, this enum is used to describe the type of a column.
///
/// Other data analytic systems or engines may support more complex and nested data types like lists, dicts, or even
/// structs that may contain [`DataType`]s, but we do not seek to support such complex types because we only focus on
/// primitive types (note that [`String`] or [`std::str`] are also primitive types in data analytics).
#[derive(
    Clone,
    Copy,
    Debug,
    Hash,
    PartialOrd,
    PartialEq,
    Eq,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromPrimitive,
    IntoPrimitive,
)]
#[repr(usize)]
pub enum DataType {
    /// Denotes data types that contain null or empty data.
    #[default]
    Null,
    /// True or false.
    Boolean,
    /// A signed 8-bit integer.
    Int8,
    /// A signed 16-bit integer.
    Int16,
    /// A signed 32-bit integer.
    Int32,
    /// A signed 64-bit integer.
    Int64,
    /// An unsigned 8-bit integer.
    UInt8,
    /// An unsigned 16-bit integer.
    UInt16,
    /// An unsigned 32-bit integer.
    UInt32,
    /// An unsigned 64-bit integer.
    UInt64,
    /// A 32-bit floating point number.
    Float32,
    /// A 64-bit floating point number.
    Float64,
    /// The UTF-8 encoded string.
    Utf8Str,
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[repr(usize)]
pub enum JoinType {
    /// LEFT OUTER JOIN
    Left,
    /// RIGHT OUTER JOIN
    Right,
    /// FULL OUTER JOIN
    Full,
    #[default]
    /// INNER JOIN
    Inner,
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[repr(usize)]
pub enum ExecutorType {
    DataframeScan,
    Filter,
    Projection,
    Apply,
    Aggregation,
    PartitionGroupBy,
    Join,
    Distinct,
    #[default]
    Invalid,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct FunctionArguments {
    /// function argument name => value
    pub inner: serde_json::Map<String, serde_json::Value>,
}

impl FunctionArguments {
    /// Gets a key from the argument and apply the transformation function, if needed.
    pub fn get_and_apply<F, T, U>(&self, key: &str, f: F) -> PolicyCarryingResult<U>
    where
        T: for<'de> Deserialize<'de>,
        F: FnOnce(T) -> U,
    {
        match self.inner.get(key).cloned() {
            Some(val) => match serde_json::from_value::<T>(val) {
                Ok(val) => Ok(f(val)),
                Err(e) => Err(PolicyCarryingError::SerializeError(e.to_string())),
            },
            None => Err(PolicyCarryingError::SerializeError(format!(
                "{key} not found"
            ))),
        }
    }
}

impl Display for DataType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for DataType {
    type Error = PolicyCarryingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "bool" | "Boolean" | "DataType::Boolean" => Ok(Self::Boolean),
            "i8" | "Int8" | "DataType::Int8" => Ok(Self::Int8),
            "i16" | "Int16" | "DataType::Int16" => Ok(Self::Int16),
            "i32" | "Int32" | "DataType::Int32" => Ok(Self::Int32),
            "i64" | "Int64" | "DataType::Int64" => Ok(Self::Int64),
            "u8" | "UInt8" | "DataType::UInt8" => Ok(Self::UInt8),
            "u16" | "UInt16" | "DataType::UInt16" => Ok(Self::UInt16),
            "u32" | "UInt32" | "DataType::UInt32" => Ok(Self::UInt32),
            "u64" | "UInt64" | "DataType::UInt64" => Ok(Self::UInt64),
            "f32" | "Float32" | "DataType::Float32" => Ok(Self::Float32),
            "f64" | "Float64" | "DataType::Float64" => Ok(Self::Float64),
            "null" | "Null" | "DataType::Null" => Ok(Self::Null),
            "str" | "String" | "Utf8Str" | "DataType::Utf8Str" => Ok(Self::Utf8Str),
            _ => Err(PolicyCarryingError::ParseError(
                "".into(),
                format!("cannot parse {value}"),
            )),
        }
    }
}

impl TryFrom<String> for DataType {
    type Error = PolicyCarryingError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl DataType {
    #[inline]
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float() || self.is_unsigned_integer()
    }

    #[inline]
    pub fn is_utf8(&self) -> bool {
        matches!(self, Self::Utf8Str)
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float32 | Self::Float64)
    }

    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Int8 | Self::Int16 | Self::Int32 | Self::Int64)
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Self::UInt8 | Self::UInt16 | Self::UInt32 | Self::UInt64
        )
    }

    pub fn to_qualified_str(&self) -> String {
        format!("DataType::{self}")
    }

    /// Casts to the upper type. For example, all integers are coerced to `Self::Int64`.
    pub fn to_upper(&self) -> Self {
        if self.is_float() {
            Self::Float64
        } else if self.is_integer() {
            Self::Int64
        } else if self.is_unsigned_integer() {
            Self::UInt64
        } else {
            *self
        }
    }
}

pub trait TypeCoerce {
    fn try_coerce(&self, to: DataType) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
        unimplemented!("cannot coerce to {to:?}")
    }
}

/// This trait is a workaround for getting the concrete type of a primitive type that we store
/// as a trait object `dyn PritimiveDataType`.
#[typetag::serde(tag = "primitive_data")]
pub trait PrimitiveDataType: TypeCoerce + Debug + Sync + Send + ToString + 'static {
    fn data_type(&self) -> DataType;

    fn as_any_ref(&self) -> &dyn Any;

    fn eq_impl(&self, other: &dyn PrimitiveDataType) -> bool;

    fn ord_impl(&self, other: &dyn PrimitiveDataType) -> Option<Ordering>;

    fn clone_box(&self) -> Box<dyn PrimitiveDataType>;
}

pub trait PrimitiveData: PrimitiveDataType + Arithmetic {
    type Native: Sized;

    const DATA_TYPE: DataType;
}

pub trait Arithmetic: Sized {
    fn zero() -> Self;

    fn add(&self, _other: &Self) -> Self {
        panic!("this type cannot be added");
    }

    fn sub(&self, _other: &Self) -> Self {
        panic!("this type cannot be added");
    }

    fn mul(&self, _other: &Self) -> Self {
        panic!("this type cannot be added");
    }

    fn div(&self, _other: &Self) -> Self {
        panic!("this type cannot be added");
    }
}

impl dyn PrimitiveDataType {
    /// Cast to `T`.
    #[inline]
    pub fn try_cast<T>(&self) -> Option<T>
    where
        T: Clone + 'static,
    {
        self.as_any_ref().downcast_ref::<T>().cloned()
    }
}

impl Eq for Box<dyn PrimitiveDataType> {}

impl Hash for Box<dyn PrimitiveDataType> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.data_type() {
            DataType::Null => u128::MAX.hash(state),
            DataType::Boolean => self.try_cast::<bool>().unwrap().hash(state),
            DataType::UInt8 => self.try_cast::<i8>().unwrap().hash(state),
            DataType::UInt16 => self.try_cast::<i16>().unwrap().hash(state),
            DataType::UInt32 => self.try_cast::<i32>().unwrap().hash(state),
            DataType::UInt64 => self.try_cast::<i64>().unwrap().hash(state),
            DataType::Int8 => self.try_cast::<i8>().unwrap().hash(state),
            DataType::Int16 => self.try_cast::<i16>().unwrap().hash(state),
            DataType::Int32 => self.try_cast::<i32>().unwrap().hash(state),
            DataType::Int64 => self.try_cast::<i64>().unwrap().hash(state),
            DataType::Float32 => {
                let num = OrderedFloat::from(self.try_cast::<f32>().unwrap());
                num.hash(state)
            }
            DataType::Float64 => {
                let num = OrderedFloat::from(self.try_cast::<f64>().unwrap());
                num.hash(state)
            }
            DataType::Utf8Str => self.try_cast::<String>().unwrap().hash(state),
        }
    }
}

impl Clone for Box<dyn PrimitiveDataType> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for dyn PrimitiveDataType {
    fn eq(&self, other: &Self) -> bool {
        self.eq_impl(other)
    }
}

impl PartialOrd for dyn PrimitiveDataType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ord_impl(other)
    }
}

#[macro_export]
macro_rules! impl_type {
    ($name:ty, $ty:expr) => {
        impl $crate::types::PrimitiveData for $name {
            type Native = $name;

            const DATA_TYPE: DataType = $ty;
        }

        #[typetag::serde]
        impl $crate::types::PrimitiveDataType for $name {
            fn data_type(&self) -> $crate::types::DataType {
                $ty
            }

            fn as_any_ref(&self) -> &dyn std::any::Any {
                self
            }

            fn eq_impl(&self, other: &dyn $crate::types::PrimitiveDataType) -> bool {
                let other_downcast = match other.as_any_ref().downcast_ref::<$name>() {
                    Some(value) => value,
                    // Not the same type
                    None => return false,
                };

                self == other_downcast
            }

            fn ord_impl(&self, other: &dyn $crate::types::PrimitiveDataType) -> Option<Ordering> {
                match other.as_any_ref().downcast_ref::<$name>() {
                    Some(value) => self.partial_cmp(&value),
                    None => None,
                }
            }

            fn clone_box(&self) -> Box<dyn PrimitiveDataType> {
                Box::new(self.clone())
            }
        }

        impl std::borrow::Borrow<dyn $crate::types::PrimitiveDataType> for $name {
            fn borrow(&self) -> &dyn $crate::types::PrimitiveDataType {
                self
            }
        }
    };
}

macro_rules! impl_numeric {
    ($name:ty) => {
        impl TypeCoerce for $name {
            fn try_coerce(&self, to: DataType) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
                match to {
                    DataType::Int8 => Ok(Box::new(self.clone() as i8)),
                    DataType::Int16 => Ok(Box::new(self.clone() as i16)),
                    DataType::Int32 => Ok(Box::new(self.clone() as i32)),
                    DataType::Int64 => Ok(Box::new(self.clone() as i64)),
                    DataType::UInt8 => Ok(Box::new(self.clone() as u8)),
                    DataType::UInt16 => Ok(Box::new(self.clone() as u16)),
                    DataType::UInt32 => Ok(Box::new(self.clone() as u32)),
                    DataType::UInt64 => Ok(Box::new(self.clone() as u64)),
                    DataType::Float32 => Ok(Box::new(self.clone() as f32)),
                    DataType::Float64 => Ok(Box::new(self.clone() as f64)),
                    // Ignored.
                    ty => Err(PolicyCarryingError::OperationNotSupported(format!(
                        "cannot cast {:?} to {:?}",
                        self.data_type(),
                        ty
                    ))),
                }
            }
        }

        impl Arithmetic for $name {
            fn zero() -> Self {
                Zero::zero()
            }

            fn add(&self, other: &Self) -> Self {
                std::ops::Add::add(self, other)
            }

            fn sub(&self, other: &Self) -> Self {
                std::ops::Sub::sub(self, other)
            }

            fn mul(&self, other: &Self) -> Self {
                std::ops::Mul::mul(self, other)
            }

            fn div(&self, other: &Self) -> Self {
                std::ops::Div::div(self, other)
            }
        }
    };
}

impl_type!(bool, DataType::Boolean);
impl_type!(i8, DataType::Int8);
impl_type!(i16, DataType::Int16);
impl_type!(i32, DataType::Int32);
impl_type!(i64, DataType::Int64);
impl_type!(u8, DataType::UInt8);
impl_type!(u16, DataType::UInt16);
impl_type!(u32, DataType::UInt32);
impl_type!(u64, DataType::UInt64);
impl_type!(f32, DataType::Float32);
impl_type!(f64, DataType::Float64);
impl_type!(String, DataType::Utf8Str);

impl_numeric!(i8);
impl_numeric!(i16);
impl_numeric!(i32);
impl_numeric!(i64);
impl_numeric!(u8);
impl_numeric!(u16);
impl_numeric!(u32);
impl_numeric!(u64);
impl_numeric!(f32);
impl_numeric!(f64);

impl TypeCoerce for bool {}
impl TypeCoerce for String {}

impl Arithmetic for bool {
    fn zero() -> Self {
        false
    }
}

impl Arithmetic for String {
    fn zero() -> Self {
        "".into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn type_correct() {
        let int8_data1: Box<dyn PrimitiveDataType> = Box::new(0);
        let int8_data2: Box<dyn PrimitiveDataType> = Box::new(100);
        let int8_data3: Box<dyn PrimitiveDataType> = Box::new(0);
        let int64_data4: Box<dyn PrimitiveDataType> = Box::new(0i64);

        assert!(&int8_data1 != &int8_data2);
        assert!(&int8_data3 != &int64_data4);
        assert!(&int8_data1 == &int8_data3);
    }

    #[test]
    fn type_serde() {
        let int8_data: Box<dyn PrimitiveDataType> = Box::new(0i8);
        let str_data: Box<dyn PrimitiveDataType> = Box::new("0".to_string());

        let serialized_int8 = serde_json::to_string(&int8_data).unwrap();
        let serialized_str = serde_json::to_string(&str_data).unwrap();

        assert_eq!(r#"{"primitive_data":"i8","value":0}"#, &serialized_int8);
        assert_eq!(
            r#"{"primitive_data":"String","value":"0"}"#,
            &serialized_str
        );
    }
}
