use std::{fmt::Debug, ops::Index, sync::Arc};

use policy_core::{error::PolicyCarryingResult, pcd_ensures, types::PrimitiveDataType};

use crate::{field::FieldRef, DataFrame};

impl DataFrame {
    /// Takes a schema of the columnar data and converts the column-oriented data into row-oriented
    /// data using data conversion.
    ///
    /// [`RowReader`] must be constructed by the policy-compliant API set to perform the necessary
    /// checks on the data the untrusted entities are trying to access.
    pub(crate) fn convert_rows(&self) -> PolicyCarryingResult<RowSet> {
        if self.columns.is_empty() {
            return Ok(RowSet::new(Vec::new()));
        }

        // Check if length is correct.
        let lengths = self.columns.iter().map(|v| v.len()).collect::<Vec<_>>();
        pcd_ensures!(
            lengths.len() <= 1 || lengths.iter().all(|&v| v == lengths[0]),
            ImpossibleOperation: "not all columns have the same length; length vec is {lengths:?}",
        );

        // Cast all columns to their concrete `FieldDataArray<T>` types.
        // FIXME: Handle null case? Currently we do not support nullable values.
        let row_count = lengths[0];
        let mut rows = RowSet::new(self.columns.iter().map(|e| e.field()).collect());
        for i in 0..row_count {
            let mut row = Vec::<Arc<dyn PrimitiveDataType>>::new();

            for column in self.columns.iter() {
                // Try to convert the data to its concrete type and return its trait object.
                let data = column.index(i)?;
                row.push(data.into());
            }

            rows.rows.push(Row { row_data: row });
        }

        Ok(rows)
    }
}

pub struct RowSet {
    pub(crate) schema: Vec<FieldRef>,
    pub(crate) rows: Vec<Row>,
}

impl RowSet {
    #[inline]
    pub fn new(schema: Vec<FieldRef>) -> Self {
        Self {
            schema,
            rows: Vec::new(),
        }
    }
}

/// A two-dimensional row of column-oriented data with a defined
/// [schema](crate::schema::Schema).
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    row_data: Vec<Arc<dyn PrimitiveDataType>>,
}

impl Index<usize> for Row {
    type Output = Arc<dyn PrimitiveDataType>;

    /// Allows the user to index the row.
    fn index(&self, index: usize) -> &Self::Output {
        &self.row_data[index]
    }
}

impl Row {
    /// Returns the stringified values of the row.
    pub fn stringify(&self) -> Vec<String> {
        self.row_data
            .iter()
            .map(|value| value.to_string())
            .collect()
    }
}
