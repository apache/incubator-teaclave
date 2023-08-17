use std::sync::Arc;

use hashbrown::{hash_map::Entry, HashMap};
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    pcd_ensures,
    types::DataType,
};

use crate::{
    field::{new_empty, Field, FieldDataArray, FieldDataRef},
    DataFrame,
};

/// Indexes of the groups, the first index is stored separately.
/// this make sorting fast.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GroupsIdx {
    pub(crate) sorted: bool,
    /// The groupby key.
    pub(crate) first: Vec<usize>,
    /// The elements grouped in the group.
    pub(crate) all: Vec<Vec<usize>>,
}

impl GroupsIdx {
    #[inline]
    pub fn groupby_with_null(&self) -> bool {
        self.first.len() == 1 && self.first[0] == 0 && self.all.len() == 1
    }

    #[inline]
    pub fn first(&self) -> &[usize] {
        &self.first
    }
}

/// A group slice since groups are 2-dimensional arrays of the original dataframe.
///
/// For each element in the slice, the first element denotes the start of the group,
/// and the second one denotes the length of the group.
pub type GroupsSlice = Vec<[usize; 2]>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupsProxy {
    Idx(GroupsIdx),
    /// A slice over groups.
    Slice(GroupsSlice),
}

impl Default for GroupsProxy {
    fn default() -> Self {
        Self::Idx(Default::default())
    }
}

impl GroupsProxy {
    pub fn len(&self) -> usize {
        match self {
            // Groupby with keys.
            GroupsProxy::Idx(groups) => groups.first.len(),
            GroupsProxy::Slice(groups) => groups.len(),
        }
    }

    /// Returns the row count as a [`FieldDataRef`].
    pub fn row_count(&self) -> PolicyCarryingResult<FieldDataRef> {
        match self {
            GroupsProxy::Idx(groups) => {
                let field = Field::new(
                    "COUNT(*)".into(),
                    DataType::UInt64,
                    false,
                    Default::default(),
                );

                let mut data =
                    FieldDataArray::from_iter(groups.all.iter().map(|idx| idx.len() as u64));
                data.field = field.into();

                Ok(Arc::new(data))
            }
            GroupsProxy::Slice(_) => {
                todo!()
            }
        }
    }
}

/// A helper struct for performing the `groupby` operation on a given dataframe.
#[derive(Clone, Debug)]
pub struct GroupByHelper<'a> {
    pub df: &'a DataFrame,
    /// [first idx, [other idx]]
    pub proxy: GroupsProxy,
    pub selected_keys: Vec<FieldDataRef>,
    /// Column names selected for aggregation.
    pub selected_aggs: Option<Vec<String>>,
}

impl<'a> GroupByHelper<'a> {
    pub fn new(
        df: &'a DataFrame,
        proxy: GroupsProxy,
        selected_keys: Vec<FieldDataRef>,
        selected_aggs: Option<Vec<String>>,
    ) -> Self {
        Self {
            df,
            proxy,
            selected_aggs,
            selected_keys,
        }
    }

    pub fn keys_sliced(&self, slice: Option<(usize, usize)>) -> Vec<FieldDataRef> {
        let groups = match slice {
            Some((offset, len)) => {
                unimplemented!("slice ({offset}, {len}) is not supported at this moment")
            }
            None => &self.proxy,
        };

        let mut ans = vec![];
        for key in self.selected_keys.iter() {
            match groups {
                GroupsProxy::Idx(groups) => {
                    let mut cur = new_empty(key.field());
                    for &idx in groups.first.iter() {
                        cur.push_erased(key.index(idx).unwrap());
                    }
                    ans.push(cur.into());
                }
                GroupsProxy::Slice(_) => unimplemented!("slice is not supported at this moment"),
            }
        }
        ans
    }
}

impl DataFrame {
    /// The same as the function in polars implementation.
    pub fn groupby_with_keys(
        &self,
        mut selected_keys: Vec<FieldDataRef>,
        maintain_order: bool,
    ) -> PolicyCarryingResult<GroupByHelper> {
        let by_len = selected_keys.len();

        if by_len == 0 {
            // If the keys are empty, then it means we have explicitly constructed a dummy
            // `groupby` clause to unify the aggregation operation. In this case, we choose
            // a phantom column `index` and group by that column.
            let height: usize = self.shape().1;

            // The `index` column is phantom since we can just create it from the height.
            // We thus do not need to manunally insert a column into the dataframe.
            let all_index = GroupsIdx {
                sorted: maintain_order,
                // `group by NULL` transforms the dataframe into a single partition.
                first: vec![0],
                all: vec![(0..height).collect()],
            };
            let group = GroupsProxy::Idx(all_index);

            Ok(GroupByHelper::new(self, group, selected_keys, None))
        } else {
            // In this case, we peek the length of the first field in the `groupby` keys.
            let by_len = selected_keys[0].len();

            // This step checks if one of the following conditions holds:
            //
            // 1. We can perform the partition operation using the `groupby` key. If there are multiple
            //    keys that are used to partition the dataframe, we use the first one.
            // 2. If the key length does not match the row number, we check if the dataframe is empty.
            //    This is because doing `groupby` on an empty dataframe is always valid, although this
            //    operation does nothing.
            if by_len != self.shape().1 && self.shape().0 != 0 {
                pcd_ensures!(
                    by_len == 1,
                    ImpossibleOperation:
                    r#"`groupby` cannot be applied because the key length does not match
                    the row number of the dataframe; nor does it have length 1 to be able
                    to broadcast to the whole dataframe. The length is {}, and the shape
                    of the dataframe is {:?}"#,
                    by_len,
                    self.shape()
                );

                // Grow the `groupby` key so that we can apply it on the whole dataframe.
                selected_keys[0] = selected_keys[0].new_from_index(0, self.shape().1);
            }

            let groups = match selected_keys.len() {
                1 => self.groupby_single_key(selected_keys[0].name(), maintain_order),
                len if len != 0 => Err(PolicyCarryingError::OperationNotSupported(
                    "`groupby` with multiple keys are not supported now".into(),
                )),
                _ => panic!("internal logic error; `len == 0` should be handled before"),
            }?;

            Ok(GroupByHelper::new(self, groups, selected_keys, None))
        }
    }

    /// Creates a lazily grouped dataframe using a single key.
    fn groupby_single_key(
        &self,
        key: &str,
        maintain_order: bool,
    ) -> PolicyCarryingResult<GroupsProxy> {
        // Iterate over the columns and combine the rows with the same keys indicated by `keys`.
        let this_column = match self
            .columns
            .iter()
            .find(|col| col.name() == key)
            .cloned()
        {
            Some(this_column) => this_column,
            None => {
                return Err(PolicyCarryingError::ColumnNotFound(format!(
                    "unable to find the column named {}",
                    key
                )))
            }
        };

        let mut map = HashMap::<_, (usize, Vec<usize>)>::new();
        for i in 0..this_column.len() {
            let cur = this_column.index(i)?;

            match map.entry(cur.clone()) {
                Entry::Occupied(mut entry) => entry.get_mut().1.push(i),
                Entry::Vacant(entry) => {
                    entry.insert((i, vec![i]));
                }
            }
        }

        let ans = map.into_values().unzip();
        let idx = GroupsIdx {
            sorted: maintain_order,
            first: ans.0,
            all: ans.1,
        };
        Ok(GroupsProxy::Idx(idx))
    }
}
