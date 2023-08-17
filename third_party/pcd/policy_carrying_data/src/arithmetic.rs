use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
};

use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::GroupByMethod,
    types::PrimitiveDataType,
    types::*,
};

use crate::field::{new_empty, FieldData, FieldDataArray, FieldDataRef, FieldRef};

macro_rules! impl_operator {
    ($op:ident, $func:ident) => {
        impl<'a, T> $op<&'a FieldDataArray<T>> for &'a FieldDataArray<T>
        where
            T: PrimitiveData
                + Debug
                + Default
                + Send
                + Sync
                + Clone
                + PartialEq
                + $op<T, Output = T>,
        {
            type Output = FieldDataArray<T>;

            fn $func(self, rhs: &'a FieldDataArray<T>) -> Self::Output {
                let new_vec = self
                    .iter()
                    .zip(rhs.iter())
                    .map(|(lhs, rhs)| lhs.clone().$func(rhs.clone()))
                    .collect();

                Self::Output::new(self.field.clone(), new_vec)
            }
        }

        /// A workaround for the implementation of arithmetic operations on the references to the dynamic trait objects.
        impl<'a> $op<&'a dyn FieldData> for &'a dyn FieldData {
            type Output = FieldDataRef;

            fn $func(self, rhs: &'a dyn FieldData) -> Self::Output {
                // Currently we only allow numeric types.
                // TODO: Type corecion; add other operators.
                if self.data_type() == rhs.data_type() {
                    // Downcast to concrete types.
                    match self.data_type() {
                        DataType::UInt8 => (self
                            .try_cast::<u8>()
                            .unwrap()
                            .$func(rhs.try_cast::<u8>().unwrap()))
                        .clone_arc(),
                        DataType::UInt16 => (self
                            .try_cast::<u16>()
                            .unwrap()
                            .$func(rhs.try_cast::<u16>().unwrap()))
                        .clone_arc(),
                        DataType::UInt32 => (self
                            .try_cast::<u32>()
                            .unwrap()
                            .$func(rhs.try_cast::<u32>().unwrap()))
                        .clone_arc(),
                        DataType::UInt64 => (self
                            .try_cast::<u64>()
                            .unwrap()
                            .$func(rhs.try_cast::<u64>().unwrap()))
                        .clone_arc(),
                        DataType::Int8 => (self
                            .try_cast::<i8>()
                            .unwrap()
                            .$func(rhs.try_cast::<i8>().unwrap()))
                        .clone_arc(),
                        DataType::Int16 => (self
                            .try_cast::<i16>()
                            .unwrap()
                            .$func(rhs.try_cast::<i16>().unwrap()))
                        .clone_arc(),
                        DataType::Int32 => (self
                            .try_cast::<i32>()
                            .unwrap()
                            .$func(rhs.try_cast::<i32>().unwrap()))
                        .clone_arc(),
                        DataType::Int64 => (self
                            .try_cast::<i64>()
                            .unwrap()
                            .$func(rhs.try_cast::<i64>().unwrap()))
                        .clone_arc(),
                        DataType::Float32 => (self
                            .try_cast::<f32>()
                            .unwrap()
                            .$func(rhs.try_cast::<f32>().unwrap()))
                        .clone_arc(),
                        DataType::Float64 => (self
                            .try_cast::<f64>()
                            .unwrap()
                            .$func(rhs.try_cast::<f64>().unwrap()))
                        .clone_arc(),
                        ty => panic!("should not go here for {ty:?}"),
                    }
                } else {
                    self.clone_arc()
                }
            }
        }
    };
}

impl_operator!(Add, add);
impl_operator!(Sub, sub);
impl_operator!(Mul, mul);
impl_operator!(Div, div);

pub(crate) fn do_aggregate<T>(
    partitions: Vec<Vec<T>>,
    field: FieldRef,
    how: GroupByMethod,
) -> PolicyCarryingResult<FieldDataRef>
where
    T: PrimitiveData + PartialOrd + Debug + Default + Send + Sync + Clone + 'static,
{
    let mut res = new_empty(field);

    for partition in partitions.into_iter() {
        let cur: Box<dyn PrimitiveDataType> = match how {
            GroupByMethod::Min => Box::new(
                partition
                    .into_iter()
                    .min_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap())
                    .ok_or(PolicyCarryingError::ImpossibleOperation(
                        "cannot find max value".into(),
                    ))?,
            ),
            GroupByMethod::Max => Box::new(
                partition
                    .into_iter()
                    .max_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap())
                    .ok_or(PolicyCarryingError::ImpossibleOperation(
                        "cannot find max value".into(),
                    ))?,
            ),
            GroupByMethod::Sum => {
                let mut sum = Box::new(
                    partition
                        .into_iter()
                        .fold(T::zero(), |acc, cur| acc.add(&cur)),
                ) as Box<dyn PrimitiveDataType>;
                sum = sum.try_coerce(res.data_type())?;
                sum
            }
            _ => {
                return Err(PolicyCarryingError::OperationNotSupported(format!(
                    "aggregation method {how:?} is not supported"
                )))
            }
        };

        res.push_erased(cur);
    }

    Ok(res.into())
}

/// By default we use `f64` to prevent overflow.
pub fn erased_sum(input: &dyn FieldData) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
    let res: Box<dyn PrimitiveDataType> = match input.data_type() {
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            Box::new(sum_impl(input.try_cast::<u64>()?, 0u64, None)?)
        }
        DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
            Box::new(sum_impl(input.try_cast::<i64>()?, 0i64, None)?)
        }
        DataType::Float32 => Box::new(sum_impl(input.try_cast::<f32>()?, 0.0f32, None)?),
        DataType::Float64 => Box::new(sum_impl(input.try_cast::<f64>()?, 0.0f64, None)?),
        ty => {
            return Err(PolicyCarryingError::OperationNotSupported(format!(
                "{ty:?}"
            )))
        }
    };

    Ok(res)
}

pub fn erased_max(input: &dyn FieldData) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
    match input.data_type() {
        DataType::UInt8 => Ok(Box::new(max_impl(input.try_cast::<u8>()?)?)),
        DataType::UInt16 => Ok(Box::new(max_impl(input.try_cast::<u16>()?)?)),
        DataType::UInt32 => Ok(Box::new(max_impl(input.try_cast::<u32>()?)?)),
        DataType::UInt64 => Ok(Box::new(max_impl(input.try_cast::<u64>()?)?)),
        DataType::Int8 => Ok(Box::new(max_impl(input.try_cast::<i8>()?)?)),
        DataType::Int16 => Ok(Box::new(max_impl(input.try_cast::<i16>()?)?)),
        DataType::Int32 => Ok(Box::new(max_impl(input.try_cast::<i32>()?)?)),
        DataType::Int64 => Ok(Box::new(max_impl(input.try_cast::<i64>()?)?)),
        DataType::Float32 => Ok(Box::new(max_impl(input.try_cast::<f32>()?)?)),
        DataType::Float64 => Ok(Box::new(max_impl(input.try_cast::<f64>()?)?)),
        ty => Err(PolicyCarryingError::OperationNotSupported(format!(
            "{ty:?}"
        ))),
    }
}

pub fn erased_min(input: &dyn FieldData) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
    match input.data_type() {
        DataType::UInt8 => Ok(Box::new(min_impl(input.try_cast::<u8>()?)?)),
        DataType::UInt16 => Ok(Box::new(min_impl(input.try_cast::<u16>()?)?)),
        DataType::UInt32 => Ok(Box::new(min_impl(input.try_cast::<u32>()?)?)),
        DataType::UInt64 => Ok(Box::new(min_impl(input.try_cast::<u64>()?)?)),
        DataType::Int8 => Ok(Box::new(min_impl(input.try_cast::<i8>()?)?)),
        DataType::Int16 => Ok(Box::new(min_impl(input.try_cast::<i16>()?)?)),
        DataType::Int32 => Ok(Box::new(min_impl(input.try_cast::<i32>()?)?)),
        DataType::Int64 => Ok(Box::new(min_impl(input.try_cast::<i64>()?)?)),
        DataType::Float32 => Ok(Box::new(min_impl(input.try_cast::<f32>()?)?)),
        DataType::Float64 => Ok(Box::new(min_impl(input.try_cast::<f64>()?)?)),
        ty => Err(PolicyCarryingError::OperationNotSupported(format!(
            "{ty:?}"
        ))),
    }
}

/// Sums up the value.
pub fn sum_impl<R, T>(
    input: &FieldDataArray<T>,
    init: R,
    upper: Option<T>,
) -> PolicyCarryingResult<R>
where
    T: PrimitiveData
        + Add<R, Output = R>
        + PartialOrd
        + Debug
        + Default
        + Send
        + Sync
        + Clone
        + 'static,
{
    // Can we really add on utf8 strings?
    if !(input.data_type().is_numeric() || input.data_type().is_utf8()) {
        Err(PolicyCarryingError::ImpossibleOperation(
            "Cannot add on non-numeric types".into(),
        ))
    } else {
        // A bad thing is, we cannot directly call `sum()` on iterator on a generic type `T`,
        // but we may call the `fold()` method to aggregate all the elements together.
        Ok(input.iter().fold(init, |acc, e| {
            let cur = match upper {
                Some(ref upper) => {
                    if upper >= e {
                        e.clone()
                    } else {
                        upper.clone()
                    }
                }
                None => e.clone(),
            };

            cur + acc
        }))
    }
}

/// Returns the maximum value of the array.
pub fn max_impl<T>(input: &FieldDataArray<T>) -> PolicyCarryingResult<T>
where
    T: PrimitiveData + PartialOrd + Debug + Default + Send + Sync + Clone + 'static,
{
    input
        .into_iter()
        .max_by(|&lhs, &rhs| lhs.partial_cmp(rhs).unwrap()) // May panic when NaN
        .cloned()
        .ok_or(PolicyCarryingError::ImpossibleOperation(
            "Input is empty".into(),
        ))
}

/// Returns the minimum value of the array.
pub fn min_impl<T>(input: &FieldDataArray<T>) -> PolicyCarryingResult<T>
where
    T: PrimitiveData + PartialOrd + Debug + Default + Send + Sync + Clone + 'static,
{
    input
        .into_iter()
        .max_by(|&lhs, &rhs| rhs.partial_cmp(lhs).unwrap()) // May panic when NaN
        .cloned()
        .ok_or(PolicyCarryingError::ImpossibleOperation(
            "Input is empty".into(),
        ))
}
