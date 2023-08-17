use std::{
    fmt::Debug,
    ops::{BitAnd, BitOr, BitXor},
};

use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    pcd_ensures,
    types::*,
};

use crate::field::{BooleanFieldData, FieldData, FieldDataArray};

macro_rules! impl_comparator {
    ($lhs:expr, $rhs:expr, $op:ident) => {
        match $lhs.data_type() {
            DataType::UInt8 => $lhs
                .try_cast::<u8>()
                .unwrap()
                .$op($rhs.try_cast::<u8>().unwrap()),
            DataType::UInt16 => $lhs
                .try_cast::<u16>()
                .unwrap()
                .$op($rhs.try_cast::<u16>().unwrap()),
            DataType::UInt32 => $lhs
                .try_cast::<u32>()
                .unwrap()
                .$op($rhs.try_cast::<u32>().unwrap()),
            DataType::UInt64 => $lhs
                .try_cast::<u64>()
                .unwrap()
                .$op($rhs.try_cast::<u64>().unwrap()),
            DataType::Int8 => $lhs
                .try_cast::<i8>()
                .unwrap()
                .$op($rhs.try_cast::<i8>().unwrap()),
            DataType::Int16 => $lhs
                .try_cast::<i16>()
                .unwrap()
                .$op($rhs.try_cast::<i16>().unwrap()),
            DataType::Int32 => $lhs
                .try_cast::<i32>()
                .unwrap()
                .$op($rhs.try_cast::<i32>().unwrap()),
            DataType::Int64 => $lhs
                .try_cast::<i64>()
                .unwrap()
                .$op($rhs.try_cast::<i64>().unwrap()),
            DataType::Float32 => $lhs
                .try_cast::<f32>()
                .unwrap()
                .$op($rhs.try_cast::<f32>().unwrap()),
            DataType::Float64 => $lhs
                .try_cast::<f64>()
                .unwrap()
                .$op($rhs.try_cast::<f64>().unwrap()),
            _ => panic!("should not go here"),
        }
    };
}

impl<T> FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + PartialOrd + 'static,
{
    /// Given a predicate, returns the corresponding boolean mask array for filtering the desired elements.
    pub fn boolean_gt(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.gt(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.gt(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                if lhs_len != rhs_len {
                    pcd_ensures!(
                        lhs_len == rhs_len,
                        ImpossibleOperation: "lengths mismatch: lhs = {}, rhs = {}",
                        lhs_len, rhs_len,
                    );
                }

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.gt(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }

    pub fn boolean_ge(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.ge(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.ge(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                if lhs_len != rhs_len {
                    return Err(PolicyCarryingError::ImpossibleOperation(format!(
                        "lengths mismatch: lhs = {}, rhs = {}",
                        lhs_len, rhs_len
                    )));
                }

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.ge(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }

    pub fn boolean_lt(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.lt(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.lt(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                pcd_ensures!(lhs_len == rhs_len,
                    ImpossibleOperation: "lengths mismatch: lhs = {}, rhs = {}", lhs_len, rhs_len);

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.lt(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }

    pub fn boolean_le(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.le(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.le(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                pcd_ensures!(lhs_len == rhs_len,
                    ImpossibleOperation: "lengths mismatch: lhs = {}, rhs = {}", lhs_len, rhs_len);

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.le(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }
}

impl<T> FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + PartialEq + 'static,
{
    pub fn boolean_eq(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.eq(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.eq(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                pcd_ensures!(lhs_len == rhs_len,
                    ImpossibleOperation: "lengths mismatch: lhs = {}, rhs = {}", lhs_len, rhs_len);

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.eq(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }

    pub fn boolean_ne(&self, other: &Self) -> PolicyCarryingResult<BooleanFieldData> {
        match (self.inner.len(), other.inner.len()) {
            // len == 1 => broadcast the predicate to all the elements of the other array.
            (_, 1) => {
                let other = &other.inner[0];
                let boolean = self.inner.iter().map(|val| (val.ne(other))).collect();

                Ok(BooleanFieldData::new(self.field.clone(), boolean))
            }

            (1, _) => {
                let this = &self.inner[0];
                let boolean = other.inner.iter().map(|val| (this.ne(val))).collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }

            (lhs_len, rhs_len) => {
                pcd_ensures!(lhs_len == rhs_len,
                    ImpossibleOperation: "lengths mismatch: lhs = {}, rhs = {}", lhs_len, rhs_len);

                let boolean = self
                    .inner
                    .iter()
                    .zip(other.inner.iter())
                    .map(|(lhs, rhs)| (lhs.ne(rhs)))
                    .collect();

                Ok(BooleanFieldData::new(other.field.clone(), boolean))
            }
        }
    }
}

/// A comparator for converting an expression like `a op b` into a boolean mask that can be further
/// used to filter out the records in a [`crate::DataFrame`]. All the member methods provided by the
/// trait have different names with [`PartialOrd`], [`PartialEq`] to prevent name ambiguity.
pub trait Comparator<T>: Send + Sync {
    type Output;

    /// Alias for `>`.
    fn gt_bool(&self, other: &T) -> Self::Output;

    /// Alias for `>=`.
    fn ge_bool(&self, other: &T) -> Self::Output;

    /// Alias for `<`.
    fn lt_bool(&self, other: &T) -> Self::Output;

    /// Alias for `<=`.
    fn le_bool(&self, other: &T) -> Self::Output;

    /// Alias for "==".
    fn eq_bool(&self, other: &T) -> Self::Output;

    /// Alias for "<>".
    fn ne_bool(&self, other: &T) -> Self::Output;
}

/// This implementation is intended to be performed directly on the dynamic trait object [`FieldData`].
impl<'a> Comparator<&'a dyn FieldData> for &'a dyn FieldData {
    type Output = PolicyCarryingResult<BooleanFieldData>;

    fn gt_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_gt)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }

    fn ge_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_ge)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }

    fn lt_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_lt)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }

    fn le_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_le)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }

    fn eq_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_eq)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }

    fn ne_bool(&self, other: &&'a dyn FieldData) -> Self::Output {
        if self.data_type().is_numeric() && other.data_type().is_numeric() {
            if self.data_type() != other.data_type() {
                Err(PolicyCarryingError::ImpossibleOperation(format!(
                    "cannot compare {} with {}",
                    self.data_type(),
                    other.data_type()
                )))
            } else {
                let mut output = impl_comparator!(self, other, boolean_ne)?;
                output.rename(self.name())?;

                Ok(output)
            }
        } else {
            Err(PolicyCarryingError::ImpossibleOperation(
                "cannot compare non-numeric types".into(),
            ))
        }
    }
}

impl BitAnd for FieldDataArray<bool> {
    type Output = PolicyCarryingResult<Self>;

    fn bitand(self, rhs: Self) -> Self::Output {
        (&self).bitand(&rhs)
    }
}

impl<'a> BitAnd<&'a FieldDataArray<bool>> for &'a FieldDataArray<bool> {
    type Output = PolicyCarryingResult<FieldDataArray<bool>>;

    fn bitand(self, rhs: Self) -> Self::Output {
        let data = self
            .into_iter()
            .zip(rhs.into_iter())
            .map(|(lhs, rhs)| (lhs & rhs))
            .collect::<Vec<_>>();

        Ok(FieldDataArray::new(self.field.clone(), data))
    }
}

impl BitOr for FieldDataArray<bool> {
    type Output = PolicyCarryingResult<Self>;

    fn bitor(self, rhs: Self) -> Self::Output {
        (&self).bitor(&rhs)
    }
}

impl<'a> BitOr<&'a FieldDataArray<bool>> for &'a FieldDataArray<bool> {
    type Output = PolicyCarryingResult<FieldDataArray<bool>>;

    fn bitor(self, rhs: Self) -> Self::Output {
        let data = self
            .into_iter()
            .zip(rhs.into_iter())
            .map(|(lhs, rhs)| (lhs | rhs))
            .collect::<Vec<_>>();

        Ok(FieldDataArray::new(self.field.clone(), data))
    }
}
impl BitXor for FieldDataArray<bool> {
    type Output = PolicyCarryingResult<Self>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        (&self).bitxor(&rhs)
    }
}

impl<'a> BitXor<&'a FieldDataArray<bool>> for &'a FieldDataArray<bool> {
    type Output = PolicyCarryingResult<FieldDataArray<bool>>;

    fn bitxor(self, rhs: Self) -> Self::Output {
        let data = self
            .into_iter()
            .zip(rhs.into_iter())
            .map(|(lhs, rhs)| (lhs ^ rhs))
            .collect::<Vec<_>>();

        Ok(FieldDataArray::new(self.field.clone(), data))
    }
}
