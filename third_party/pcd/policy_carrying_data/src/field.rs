use std::{
    any::Any,
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
    ops::{Deref, Index, Range},
    sync::Arc,
};

use hashbrown::HashMap;
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    expr::GroupByMethod,
    types::*,
};
use roaring::RoaringTreemap as Bitmap;
use serde::{Deserialize, Serialize};

use crate::{
    arithmetic::do_aggregate,
    group::{GroupsIdx, GroupsProxy},
};

pub type FieldRef = Arc<Field>;
pub type FieldDataRef = Arc<dyn FieldData>;
pub type FieldMetadata = HashMap<String, String>;

// Column data arrays.
pub type Int8FieldData = FieldDataArray<i8>;
pub type Int16FieldData = FieldDataArray<i16>;
pub type Int32FieldData = FieldDataArray<i32>;
pub type Int64FieldData = FieldDataArray<i64>;
pub type UInt8FieldData = FieldDataArray<u8>;
pub type UInt16FieldData = FieldDataArray<u16>;
pub type UInt32FieldData = FieldDataArray<u32>;
pub type UInt64FieldData = FieldDataArray<u64>;
pub type Float32FieldData = FieldDataArray<f32>;
pub type Float64FieldData = FieldDataArray<f64>;
pub type StrFieldData = FieldDataArray<String>;
pub type BooleanFieldData = FieldDataArray<bool>;

/// Represents a column/attribute in the data table which may carry some specific policies. This struct is an element in
/// the schema's ([`crate::schema::Schema`]) vector of fields.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Field {
    /// The name of the field
    pub name: String,
    /// The data type of the field
    pub data_type: DataType,
    /// Whether this field contains null
    pub nullable: bool,
    /// The metadata of the field
    pub metadata: FieldMetadata,
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.name, self.data_type)
    }
}

pub trait Aggregate {
    fn aggregate(
        &self,
        how: GroupByMethod,
        groups: &GroupsProxy,
    ) -> PolicyCarryingResult<FieldDataRef>;
}

/// This trait allows us to store various types of columns into one concrete array without all the boilerplate related
/// to the type conversion. Note however, that in our implementation, this trait is only implemented for the type
/// [`FieldDataArray<T>`], and we will frequently case between trait objects.
pub trait FieldData: Aggregate + Debug + Send + Sync {
    fn data_type(&self) -> DataType;

    /// Returns the length of the data.
    fn len(&self) -> usize;

    /// Slices using groups.
    fn slice_grouped(&self, idx: &GroupsIdx) -> FieldDataRef;

    /// Reshapes according to dimension.
    fn reshape(&self, dims: (i64, i64)) -> PolicyCarryingResult<FieldDataRef>;

    /// Allows convenient downcast conversion if we want to get the concrete type of the trait object.
    fn as_any_ref(&self) -> &dyn Any;

    /// Allows convenient downcast conversion if we want to get the concrete type of the trait object.
    fn as_mut_ref(&mut self) -> &mut dyn Any;

    /// The inner data.
    fn eq_impl(&self, other: &dyn FieldData) -> bool;

    /// Returns true if the field data is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new field data array with a given index.
    fn new_from_index(&self, idx: usize, len: usize) -> FieldDataRef;

    /// Gets the element with erased type.
    fn index(&self, idx: usize) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>>;

    /// Slices the field data array.
    fn slice(&self, range: Range<usize>) -> FieldDataRef;

    /// Gets the field.
    fn field(&self) -> FieldRef;

    /// Pushes an erased type.
    fn push_erased(&mut self, val: Box<dyn PrimitiveDataType>);

    /// To json.
    fn to_json(&self) -> String;

    /// Creates a null array.
    fn full_null(&self, len: usize) -> FieldDataRef;

    /// Gets the name.
    fn name(&self) -> &str;

    /// Rename it.
    fn rename(&mut self, name: &str) -> PolicyCarryingResult<()>;

    /// Filters by boolean mask. This operation clones data.
    fn filter(&self, boolean: &BooleanFieldData) -> PolicyCarryingResult<Arc<dyn FieldData>>;

    /// Clones itself and wraps itself into an [`std::sync::Arc`].
    fn clone_arc(&self) -> FieldDataRef;
}

impl dyn FieldData + '_ {
    /// Try to downcast the trait object to its concrete type by interpreting this as a
    /// [`std::any::Any`]. This method must not be a trait method as introductin the gene-
    /// ric type `T` would make the trait object-unsfe, and thus a lot components would
    /// break. We may still, however, want to get the concrete type to perform some nece-
    /// ssary operations such as indexing. Without casting, there is no safe way to fulfill
    /// them elegantly.
    ///
    /// # Safety
    ///
    /// Should be 'safe' as long as the caller always check the concrete type by Calling
    /// `data_type()` on it. Bypassing the [`std::any::TypeId`] check gives a way to cast
    /// between trait objects from different builds.
    pub fn try_cast<T>(&self) -> PolicyCarryingResult<&FieldDataArray<T>>
    where
        T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    {
        unsafe {
            Ok(self
                .as_any_ref()
                .downcast_ref_unchecked::<FieldDataArray<T>>())
        }
    }

    /// A similar operation as [`try_cast`] but uses a mutable borrow to `self` instead.
    pub fn try_cast_mut<T>(&mut self) -> PolicyCarryingResult<&mut FieldDataArray<T>>
    where
        T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    {
        unsafe {
            Ok(self
                .as_mut_ref()
                .downcast_mut_unchecked::<FieldDataArray<T>>())
        }
    }

    pub fn as_boolean(&self) -> PolicyCarryingResult<&FieldDataArray<bool>> {
        self.try_cast::<bool>()
    }
}

impl PartialEq for dyn FieldData + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.eq_impl(other)
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
{
    /// The field  that allows for identification of the field this array belongs to.
    pub(crate) field: FieldRef,
    /// Inner storage.
    pub(crate) inner: Vec<T>,
    /// The bitmap for bookkeeping the nullance.
    pub(crate) bitmap: Bitmap,
}

impl<T> PartialOrd for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + PartialOrd + 'static,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl<T> PartialEq for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + PartialEq + 'static,
{
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Index<usize> for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}

impl<'a, T> IntoIterator for &'a FieldDataArray<T>
where
    T: PrimitiveData + Debug + Default + Send + Sync + Clone + 'static,
{
    type Item = &'a T;

    type IntoIter = FieldDataArrayIteratorBorrow<'a, T, FieldDataArray<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
{
    type Item = T;

    type IntoIter = FieldDataArrayIterator<T, Self>;

    fn into_iter(self) -> Self::IntoIter {
        let end = self.inner.len();

        Self::IntoIter {
            access: self,
            cur: 0,
            end,
            _phantom: PhantomData,
        }
    }
}

/// Iterator that allows to iterate over the array.
pub struct FieldDataArrayIterator<T, A>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    A: ArrayAccess,
{
    access: A,
    cur: usize,
    end: usize,
    _phantom: PhantomData<T>,
}

/// Iterator that allows to iterate over the array.
pub struct FieldDataArrayIteratorBorrow<'a, T, A>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    A: ArrayAccess,
{
    access: &'a A,
    cur: usize,
    end: usize,
    _phantom: PhantomData<T>,
}

impl<T, A> Iterator for FieldDataArrayIterator<T, A>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    A: ArrayAccess,
{
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur >= self.end {
            true => None,
            false => {
                let item = match self.access.index_data(self.cur) {
                    Some(item) => item,
                    None => return None,
                };
                self.cur += 1;
                Some(item.clone())
            }
        }
    }
}

impl<'a, T, A> Iterator for FieldDataArrayIteratorBorrow<'a, T, A>
where
    T: PrimitiveData + Debug + Send + Sync + Clone + 'static,
    A: ArrayAccess,
{
    type Item = &'a A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur >= self.end {
            true => None,
            false => {
                let item = match self.access.index_data(self.cur) {
                    Some(item) => item,
                    None => return None,
                };
                self.cur += 1;
                Some(item)
            }
        }
    }
}

/// The access helper for the field data array that can be used to construct iterators over arrays with zero-copy.
///
/// This feature is created as a trait because the array access behavior may vary with different types of the array.
pub trait ArrayAccess {
    type Item: Clone;

    /// Reads the index `idx` and returns [`Some`] if the index is within the range.
    fn index_data(&self, idx: usize) -> Option<&Self::Item>;
}

impl<T> Aggregate for FieldDataArray<T>
where
    T: PrimitiveData
        + Serialize
        + Debug
        + Default
        + Send
        + Sync
        + Clone
        + PartialEq
        + PartialOrd
        + 'static,
{
    fn aggregate(
        &self,
        how: GroupByMethod,
        groups: &GroupsProxy,
    ) -> PolicyCarryingResult<FieldDataRef> {
        if let GroupsProxy::Idx(idx) = groups {
            // Note that doing aggregation will change the type of the resulting field if needed.
            let all_partitions = idx
                .all
                .iter()
                .map(|index| index.iter().map(|&idx| self.inner[idx].clone()).collect())
                .collect::<Vec<Vec<_>>>();

            let mut field = self.field.deref().clone();
            field.name = format!("{:?}({})", how, field.name);
            if how.need_coerce() {
                field.data_type = field.data_type.to_upper();
            }

            do_aggregate(all_partitions, field.into(), how)
        } else {
            Err(PolicyCarryingError::OperationNotSupported(
                "cannot aggregate on a sliced groups proxy at this moment".into(),
            ))
        }
    }
}

impl<T> FieldData for FieldDataArray<T>
where
    T: PrimitiveData
        + Serialize
        + Debug
        + Default
        + Send
        + Sync
        + Clone
        + PartialEq
        + PartialOrd
        + 'static,
{
    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_mut_ref(&mut self) -> &mut dyn Any {
        self
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn slice_grouped(&self, idx: &GroupsIdx) -> FieldDataRef {
        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter(|(index, _)| idx.first().contains(index))
            .map(|(_, data)| data.clone())
            .collect::<Vec<_>>();

        let mut bitmap = Bitmap::new();
        bitmap.insert_range(0..inner.len() as u64);

        Arc::new(FieldDataArray {
            field: self.field.clone(),
            inner,
            bitmap,
        })
    }

    fn data_type(&self) -> DataType {
        T::DATA_TYPE
    }

    fn name(&self) -> &str {
        &self.field.name
    }

    fn full_null(&self, len: usize) -> FieldDataRef {
        Arc::new(Self::new_null(self.field.clone(), len))
    }

    fn push_erased(&mut self, val: Box<dyn PrimitiveDataType>) {
        let val = val.as_any_ref().downcast_ref::<T>().cloned().unwrap();
        self.inner.push(val)
    }

    fn new_from_index(&self, idx: usize, len: usize) -> FieldDataRef {
        let mut bitmap = Bitmap::new();
        bitmap.insert_range(0..len as u64);

        Arc::new(Self {
            field: self.field.clone(),
            inner: vec![self.inner[idx].clone(); len],
            bitmap,
        })
    }

    fn index(&self, idx: usize) -> PolicyCarryingResult<Box<dyn PrimitiveDataType>> {
        self.inner
            .get(idx)
            .ok_or(PolicyCarryingError::OutOfBound(format!(
                "The index {idx} is out of bound",
            )))
            .map(|val| val.clone_box())
    }

    fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    fn slice(&self, range: Range<usize>) -> FieldDataRef {
        let mut bitmap = Bitmap::new();
        bitmap.insert_range(range.start as u64..range.end as u64);

        Arc::new(Self {
            field: self.field.clone(),
            inner: self.inner[range].to_vec(),
            bitmap,
        })
    }

    #[allow(unused)]
    fn reshape(&self, dims: (i64, i64)) -> PolicyCarryingResult<FieldDataRef> {
        todo!()
    }

    fn rename(&mut self, name: &str) -> PolicyCarryingResult<()> {
        self.field = Arc::new(Field {
            name: name.into(),
            data_type: self.field.data_type,
            nullable: self.field.nullable,
            metadata: Default::default(),
        });

        Ok(())
    }

    fn field(&self) -> FieldRef {
        self.field.clone()
    }

    fn eq_impl(&self, other: &dyn FieldData) -> bool {
        if self.data_type() != other.data_type() {
            false
        } else {
            let arr = match other.as_any_ref().downcast_ref::<FieldDataArray<T>>() {
                Some(arr) => arr,
                None => return false,
            };

            self.inner == arr.inner
        }
    }

    fn filter(&self, boolean: &BooleanFieldData) -> PolicyCarryingResult<Arc<dyn FieldData>> {
        // Check if length matches.
        if boolean.len() != self.len() {
            return Err(PolicyCarryingError::ImpossibleOperation(format!(
                "length mismatch, expected {}, got {}",
                boolean.len(),
                self.len()
            )));
        }

        let inner = self
            .inner
            .iter()
            .enumerate()
            .filter(|(idx, _)| boolean.inner[*idx])
            .map(|(_, v)| v)
            .cloned()
            .collect();

        Ok(Arc::new(FieldDataArray::new(self.field.clone(), inner)))
    }

    fn clone_arc(&self) -> FieldDataRef {
        Arc::new(self.clone())
    }
}

impl<T> Debug for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.inner).finish()
    }
}

impl<T> ArrayAccess for FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone,
{
    type Item = T;

    fn index_data(&self, idx: usize) -> Option<&Self::Item> {
        self.inner.get(idx)
    }
}

impl<'a, T> ArrayAccess for &'a FieldDataArray<T>
where
    T: PrimitiveData + Debug + Send + Sync + Clone,
{
    type Item = T;

    fn index_data(&self, idx: usize) -> Option<&Self::Item> {
        self.inner.get(idx)
    }
}

impl<T> FromIterator<T> for FieldDataArray<T>
where
    T: PrimitiveData + Default + Debug + Send + Sync + Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let inner = iter.into_iter().collect::<Vec<_>>();
        let mut bitmap = Bitmap::new();
        bitmap.insert_range(0..inner.len() as u64);
        Self {
            field: Default::default(),
            inner,
            bitmap,
        }
    }
}

impl<T> FieldDataArray<T>
where
    T: PrimitiveData + Default + Debug + Send + Sync + Clone,
{
    #[inline]
    pub fn new(field: FieldRef, inner: Vec<T>) -> Self {
        let mut bitmap = Bitmap::new();
        bitmap.insert_range(0..inner.len() as u64);

        Self {
            field,
            inner,
            bitmap,
        }
    }

    #[inline]
    pub fn new_empty(field: FieldRef) -> Self {
        Self {
            field,
            inner: Vec::new(),
            bitmap: Bitmap::new(),
        }
    }

    /// Creates a null array.
    pub fn new_null(field: FieldRef, len: usize) -> Self {
        Self {
            field,
            inner: vec![Default::default(); len],
            // create an empty bitmap.
            bitmap: Bitmap::new(),
        }
    }

    pub fn data_type(&self) -> DataType {
        T::DATA_TYPE
    }

    /// Performs slicing on a field data array and returns a cloned `Self`.
    pub fn slice(&self, range: Range<usize>) -> Option<Self> {
        // Check if the boundary is correct.
        if range.start >= self.inner.len() || range.end - range.start > self.inner.len() {
            None
        } else {
            Some(Self::new(self.field.clone(), self.inner[range].to_vec()))
        }
    }

    /// Creates a new array with `item` duplicated `num` times.
    pub fn new_with_duplicate(item: T, num: usize, name: String) -> Self {
        let mut bitmap = Bitmap::new();
        bitmap.insert_range(0..num as u64);

        Self {
            field: Arc::new(Field {
                name,
                data_type: T::DATA_TYPE,
                nullable: false,
                metadata: Default::default(),
            }),
            inner: vec![item.clone(); num],
            bitmap,
        }
    }

    /// Returns an iterator on borrowed array.
    pub fn iter(&self) -> FieldDataArrayIteratorBorrow<T, Self> {
        let end = self.inner.len();

        FieldDataArrayIteratorBorrow {
            access: self,
            cur: 0,
            end,
            _phantom: PhantomData,
        }
    }
}

impl PartialEq for Field {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.data_type == other.data_type
            && self.nullable == other.nullable
    }
}

impl Eq for Field {}

impl Field {
    pub fn new(name: String, data_type: DataType, nullable: bool, metadata: FieldMetadata) -> Self {
        Self {
            name,
            data_type,
            nullable,
            metadata,
        }
    }

    pub fn new_literal(data_type: DataType) -> Self {
        Self {
            name: "Literal".into(),
            data_type,
            ..Default::default()
        }
    }

    /// Checks if the field has no name.
    #[inline]
    pub fn is_anonymous(&self) -> bool {
        self.name.is_empty()
    }
}

#[macro_export]
macro_rules! define_from_arr {
    ($name:ident, $primitive:ident, $data_type:path) => {
        impl From<Vec<$primitive>> for $name {
            fn from(other: Vec<$primitive>) -> Self {
                let mut field = Field::default();
                field.data_type = $data_type;

                Self::new(FieldRef::new(field), other)
            }
        }

        impl From<&[$primitive]> for $name {
            fn from(other: &[$primitive]) -> Self {
                Self::new(Default::default(), other.to_vec())
            }
        }
    };
}

impl From<&[&str]> for StrFieldData {
    fn from(value: &[&str]) -> Self {
        Self::new(
            Default::default(),
            value.iter().map(|v| v.to_string()).collect(),
        )
    }
}

impl From<Vec<&str>> for StrFieldData {
    fn from(value: Vec<&str>) -> Self {
        Self::new(
            Default::default(),
            value.iter().map(|v| v.to_string()).collect(),
        )
    }
}

/// Creates an empty [`FieldDataArray`] and returns as a trait object.
pub fn new_empty(field: FieldRef) -> Box<dyn FieldData> {
    match field.data_type {
        DataType::Boolean => Box::new(BooleanFieldData::new_empty(field)),
        DataType::Int8 => Box::new(Int8FieldData::new_empty(field)),
        DataType::Int16 => Box::new(Int16FieldData::new_empty(field)),
        DataType::Int32 => Box::new(Int32FieldData::new_empty(field)),
        DataType::Int64 => Box::new(Int64FieldData::new_empty(field)),
        DataType::UInt8 => Box::new(UInt8FieldData::new_empty(field)),
        DataType::UInt16 => Box::new(UInt16FieldData::new_empty(field)),
        DataType::UInt32 => Box::new(UInt32FieldData::new_empty(field)),
        DataType::UInt64 => Box::new(UInt64FieldData::new_empty(field)),
        DataType::Float32 => Box::new(Float32FieldData::new_empty(field)),
        DataType::Float64 => Box::new(Float64FieldData::new_empty(field)),
        DataType::Utf8Str => Box::new(StrFieldData::new_empty(field)),

        _ => {
            unimplemented!()
        }
    }
}

/// Creates a [`FieldDataArray`] with all null objects and returns as a trait object.
pub fn new_null(field: FieldRef, len: usize) -> Box<dyn FieldData> {
    match field.data_type {
        DataType::Boolean => Box::new(BooleanFieldData::new_null(field, len)),
        DataType::Int8 => Box::new(Int8FieldData::new_null(field, len)),
        DataType::Int16 => Box::new(Int16FieldData::new_null(field, len)),
        DataType::Int32 => Box::new(Int32FieldData::new_null(field, len)),
        DataType::Int64 => Box::new(Int64FieldData::new_null(field, len)),
        DataType::UInt8 => Box::new(UInt8FieldData::new_null(field, len)),
        DataType::UInt16 => Box::new(UInt16FieldData::new_null(field, len)),
        DataType::UInt32 => Box::new(UInt32FieldData::new_null(field, len)),
        DataType::UInt64 => Box::new(UInt64FieldData::new_null(field, len)),
        DataType::Float32 => Box::new(Float32FieldData::new_null(field, len)),
        DataType::Float64 => Box::new(Float64FieldData::new_null(field, len)),
        DataType::Utf8Str => Box::new(StrFieldData::new_null(field, len)),

        _ => {
            unimplemented!()
        }
    }
}

define_from_arr!(Int8FieldData, i8, DataType::Int8);
define_from_arr!(Int16FieldData, i16, DataType::Int16);
define_from_arr!(Int32FieldData, i32, DataType::Int32);
define_from_arr!(Int64FieldData, i64, DataType::Int64);
define_from_arr!(UInt8FieldData, u8, DataType::UInt8);
define_from_arr!(UInt16FieldData, u16, DataType::UInt16);
define_from_arr!(UInt32FieldData, u32, DataType::UInt32);
define_from_arr!(UInt64FieldData, u64, DataType::UInt64);
define_from_arr!(Float32FieldData, f32, DataType::Float32);
define_from_arr!(Float64FieldData, f64, DataType::Float64);
define_from_arr!(StrFieldData, String, DataType::Utf8Str);
define_from_arr!(BooleanFieldData, bool, DataType::Boolean);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iterator_works() {
        let int8_data = Int8FieldData::from(vec![1i8, 2, 3, 4, 5]);

        for (idx, item) in int8_data.into_iter().enumerate() {
            assert_eq!(item, (idx + 1) as i8);
        }
    }

    #[test]
    fn test_trait_eq_works() {
        let int8_data_lhs: Box<dyn FieldData> =
            Box::new(Int8FieldData::from(vec![1i8, 2, 3, 4, 5]));
        let int8_data_rhs: Box<dyn FieldData> =
            Box::new(Int8FieldData::from(vec![1i8, 2, 3, 4, 5]));
        let string_data: Box<dyn FieldData> = Box::new(StrFieldData::from(vec!["foo", "bar"]));

        // Compare at the trait level.
        assert!(int8_data_lhs == int8_data_rhs);
        assert!(string_data != int8_data_lhs);
    }

    #[test]
    fn test_trait_cast() {
        let data: Box<dyn FieldData> = Box::new(Int8FieldData::from(vec![1, 2, 3, 4, 5]));

        // Compare at the trait level.
        let arr = data.try_cast::<i8>();
        assert!(arr.is_ok());
    }

    #[test]
    fn test_slice() {
        let data = Int8FieldData::from(vec![1, 2, 3, 4, 5]);
        let lhs = data.slice(0..3);
        let rhs = Int8FieldData::from(vec![1, 2, 3]);

        assert!(lhs.is_some_and(|lhs| lhs == rhs));
    }
}
