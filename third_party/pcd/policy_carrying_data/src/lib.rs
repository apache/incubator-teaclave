#![cfg_attr(not(test), deny(unused_must_use))]
#![cfg_attr(test, allow(unused))]
#![feature(downcast_unchecked)]

use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
};

use csv::Reader;
use field::{FieldData, FieldDataArray, FieldDataRef};
use hashbrown::HashSet;
use policy_core::{
    error::{PolicyCarryingError, PolicyCarryingResult},
    pcd_ensures,
};
use schema::{Schema, SchemaRef};

pub mod arithmetic;
pub mod field;
pub mod row;
pub mod schema;

mod comparator;
pub mod group;
mod macros;

pub use comparator::Comparator;
pub use macros::*;
pub use policy_core::types::{self, *};

#[cfg(feature = "prettyprint")]
pub mod pretty;

/// The concrete struct that represents the policy-carrying data. This struct is used when we want to generate policy
/// compliant APIs for a user-defined data schema. For example, say we have the following annotated struct that stands
/// for the patient diagnosis data from a hospital:
///
/// ```
/// #[policy_carrying(Allow)]
/// pub struct DiagnosisData {
///     #[allows(read)]
///     #[implements(min, max)]
///     age: u8,
/// }
/// ```
/// which will be then converted to:
///
/// 1. A policy struct:
///
///```
/// pub struct DiagnosisDataPolicy { /*...*/ }
///```
///
/// , and
///
/// 2. an execution layer that enforces the access to the data is policy-compliant:
///
/// ```
///
/// pub struct MyDataFrameScanExec(DataFrameExec);
///
/// impl PhysicalExecutor for MyDataFrameScanExec {
///     /* ... */
/// }
/// ```
///
/// where policy-compliant executors can be executed while those not allowed will trigger an error at runtime.
///
/// Note that there is no way to directly access the data because no methods are implemented for the
/// [`DataFrame`], and the function tha tries to use the confidential data for data analysis must forbid
/// `unsafe` code by the annotation `#![forbid(unsafe_code)]`.
///
/// # Lazy Evaluation
///
/// By default, the [`DataFrame`] is lazy, which means that all the potential optimizations and
/// query execution will be  performed upon the data being collected. This is similar to polars'
/// `LazyFrame` implementation. The [`LazyFrame`] can be obtained by calling [`IntoLazy::make_lazy`]
/// on the [`DataFrame`].
///
/// # Note
///
/// The policy-carrying data is still under very active development. Implementations, API definitions, and
/// crate layout may be subject to change without any notification.
#[derive(Clone, Default, PartialEq)]
pub struct DataFrame {
    /// The concrete data.
    pub(crate) columns: Vec<FieldDataRef>,
}

impl Display for DataFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for DataFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "shape: {:?}", self.shape())?;

        println!("{:?}", self.columns());
        #[cfg(feature = "prettyprint")]
        return write!(
            f,
            "{}",
            crate::pretty::print_rows(&self.convert_rows().unwrap())
        );

        #[cfg(not(feature = "prettyprint"))]
        return write!(f, "{:?}", self.data);
    }
}

impl DataFrame {
    #[inline]
    pub fn shape(&self) -> (usize, usize) {
        match self.columns.as_slice() {
            &[] => (0, 0),
            _ => (self.columns.len(), self.columns[0].len()),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        !self.columns.is_empty()
    }

    /// Do projection.
    pub fn projection<T: AsRef<str>>(&self, columns: &[T]) -> PolicyCarryingResult<Self> {
        let names = columns.into_iter().map(|c| c.as_ref()).collect::<Vec<_>>();

        Ok(Self {
            columns: self
                .columns
                .iter()
                .filter(|column| names.contains(&column.name()))
                .cloned()
                .collect(),
        })
    }

    /// Loads the CSV file into the dataframe with its schema specified by `schema`.
    pub fn load_csv(buf: &[u8], schema: Option<SchemaRef>) -> PolicyCarryingResult<Self> {
        let mut reader = Reader::from_reader(buf);

        // If this CSV file has header, we check if this matches (the subset of) the schema.
        let schema = match (reader.headers().cloned(), schema) {
            (Ok(headers), Some(schema)) => {
                let columns = schema.fields_owned();
                pcd_ensures!(headers.len() >= columns.len(),
                    SchemaMismatch: "the given schema is incompatible with the CSV");

                // Check if the schema names are incorporated in the CSV file.
                let csv_headers = HashSet::<&str>::from_iter(headers.into_iter());
                for column in columns {
                    let name = column.name.as_str();
                    pcd_ensures!(
                        csv_headers.contains(name),
                        SchemaMismatch: "the given column name {name} is not found",
                    )
                }

                schema
            }

            // We cannot produce any schema from it!
            _ => unimplemented!(
                "currently the csv must have a header, and the schema must also be specified"
            ),
        };

        let mut columns = schema.empty_field_data();
        // Iterator over the csv file records, and construct column-oriented data structure.
        for record in reader.records().into_iter() {
            if let Ok(record) = record {
                // idx is used to consult the schema for the data type.
                for (idx, data) in record.into_iter().enumerate() {
                    if let Some(field_data) = columns.get_mut(idx) {
                        match field_data.data_type() {
                            DataType::Boolean => {
                                push_type!(field_data, data, bool, bool);
                            }
                            DataType::Int8 => {
                                push_type!(field_data, data, i8, i8);
                            }
                            DataType::Int16 => {
                                push_type!(field_data, data, i16, i16);
                            }
                            DataType::Int32 => {
                                push_type!(field_data, data, i32, i32);
                            }
                            DataType::Int64 => {
                                push_type!(field_data, data, i64, i64);
                            }
                            DataType::UInt8 => {
                                push_type!(field_data, data, u8, u8);
                            }
                            DataType::UInt16 => {
                                push_type!(field_data, data, u16, u16);
                            }
                            DataType::UInt32 => {
                                push_type!(field_data, data, u32, u32);
                            }
                            DataType::UInt64 => {
                                push_type!(field_data, data, u64, u64);
                            }
                            DataType::Float32 => {
                                push_type!(field_data, data, f32, f32);
                            }
                            DataType::Float64 => {
                                push_type!(field_data, data, f64, f64);
                            }
                            DataType::Utf8Str => {
                                push_type!(field_data, data, String, String);
                            }

                            _ => (),
                        }
                    }
                }
            }
        }

        Ok(DataFrame::new_with_cols(
            columns
                .into_iter()
                .map(|column| Arc::from(column))
                .collect(),
        ))
    }

    pub fn new_with_cols(columns: Vec<FieldDataRef>) -> Self {
        Self { columns }
    }

    pub fn schema(&self) -> SchemaRef {
        Arc::new(Schema {
            fields: self.columns.iter().map(|c| c.field()).collect(),
            metadata: Default::default(),
        })
    }

    pub fn to_json(&self) -> String {
        self.columns
            .iter()
            .map(|d| d.to_json())
            .collect::<Vec<_>>()
            .join(";")
    }

    pub fn join(
        &self,
        other: &Self,
        selected_left: Vec<FieldDataRef>,
        selected_right: Vec<FieldDataRef>,
        join_type: JoinType,
    ) -> PolicyCarryingResult<Self> {
        pcd_ensures!(
            selected_left.len() == selected_right.len(),
            InvalidInput: "the join predicate should share the same length; got {} and {}", selected_right.len(),
            selected_right.len()
        );

        // Name and type should match.
        if let Some((lhs, rhs)) = selected_left
            .iter()
            .zip(selected_right.iter())
            .find(|(lhs, rhs)| lhs.data_type() != rhs.data_type())
        {
            return Err(PolicyCarryingError::ImpossibleOperation(
                format!("but there exist two columns whose names are the same but have different types: {lhs:?} and {rhs:?}")
            ));
        }

        if selected_left.len() == 1 {
            self.join_on_single(other, selected_left, selected_right, join_type)
        } else {
            self.join_on_multiple(other, selected_left, selected_right, join_type)
        }
    }

    #[allow(unused_variables)]
    fn join_on_single(
        &self,
        other: &Self,
        selected_left: Vec<FieldDataRef>,
        selected_right: Vec<FieldDataRef>,
        join_type: JoinType,
    ) -> PolicyCarryingResult<Self> {
        todo!()
    }

    #[allow(unused_variables)]
    fn join_on_multiple(
        &self,
        other: &Self,
        selected_left: Vec<FieldDataRef>,
        selected_right: Vec<FieldDataRef>,
        join_type: JoinType,
    ) -> PolicyCarryingResult<Self> {
        todo!()
    }

    pub fn from_json(content: &str) -> PolicyCarryingResult<Self> {
        let arr = content.split(";").collect::<Vec<_>>();
        let mut columns = Vec::new();

        for element in arr {
            let value = serde_json::from_str::<serde_json::Value>(element).map_err(|_| {
                PolicyCarryingError::InvalidInput("cannot recover from json".into())
            })?;
            let ty = serde_json::from_value::<DataType>(value["field"]["data_type"].clone())
                .map_err(|_| {
                    PolicyCarryingError::InvalidInput("cannot recover from json".into())
                })?;

            let column: Arc<dyn FieldData> = match ty {
                DataType::Boolean => Arc::new(
                    serde_json::from_value::<FieldDataArray<bool>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::UInt8 => Arc::new(
                    serde_json::from_value::<FieldDataArray<u8>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::UInt16 => Arc::new(
                    serde_json::from_value::<FieldDataArray<u16>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::UInt32 => Arc::new(
                    serde_json::from_value::<FieldDataArray<u32>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::UInt64 => Arc::new(
                    serde_json::from_value::<FieldDataArray<u64>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Int8 => Arc::new(
                    serde_json::from_value::<FieldDataArray<i8>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Int16 => Arc::new(
                    serde_json::from_value::<FieldDataArray<i16>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Int32 => Arc::new(
                    serde_json::from_value::<FieldDataArray<i32>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Int64 => Arc::new(
                    serde_json::from_value::<FieldDataArray<i64>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Float32 => Arc::new(
                    serde_json::from_value::<FieldDataArray<f32>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),
                DataType::Float64 => Arc::new(
                    serde_json::from_value::<FieldDataArray<f64>>(value).map_err(|_| {
                        PolicyCarryingError::InvalidInput("cannot recover from json".into())
                    })?,
                ),

                _ => unimplemented!(),
            };

            columns.push(column);
        }

        Ok(Self { columns })
    }

    /// Takes the [..head] range of the data frame.
    #[must_use]
    pub fn take_head(&self, head: usize) -> Self {
        Self {
            columns: self.columns.iter().map(|c| c.slice(0..head)).collect(),
        }
    }

    /// Takes the [tail..] range of the data frame.
    #[must_use]
    pub fn take_tail(&self, tail: usize) -> Self {
        Self {
            columns: self
                .columns
                .iter()
                .map(|c| c.slice(tail..c.len()))
                .collect(),
        }
    }

    /// Applies a boolean filter array on this dataframe and returns a new one.
    #[must_use]
    pub fn filter(&self, boolean: &FieldDataArray<bool>) -> PolicyCarryingResult<Self> {
        let data = self
            .columns
            .iter()
            .map(|v| match v.filter(boolean) {
                Ok(d) => Ok(d),
                Err(e) => Err(e),
            })
            .collect::<PolicyCarryingResult<_>>()?;

        Ok(Self::new_with_cols(data))
    }

    /// Finds a column name in the schema of this dataframe.
    pub fn find_column(&self, name: &str) -> PolicyCarryingResult<FieldDataRef> {
        self.columns
            .iter()
            .find(|col| col.name() == name)
            .map(|col| col.clone())
            .ok_or(PolicyCarryingError::ColumnNotFound(name.into()))
    }

    pub fn columns(&self) -> &[Arc<dyn FieldData>] {
        self.columns.as_ref()
    }
}

#[cfg(feature = "read-fs")]
impl TryFrom<FunctionArguments> for DataFrame {
    type Error = PolicyCarryingError;

    fn try_from(args: FunctionArguments) -> Result<Self, Self::Error> {
        let df_path = args.get_and_apply("path", |path: String| path)?;
        let schema = args
            .get_and_apply("schema", |schema: String| {
                serde_json::from_str::<Schema>(&schema)
            })?
            .unwrap();

        let buf = std::fs::read_to_string(&df_path)
            .map_err(|e| PolicyCarryingError::FsError(e.to_string()))?;
        let buf = buf.as_bytes();
        Self::load_csv(buf, Some(schema.into()))
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use crate::schema::SchemaBuilder;

    use super::*;

    #[test]
    fn test_load_csv() {
        let schema = define_schema! {
            "column_1" => DataType::Int8,
            "column_2" => DataType::Float64,
        };

        let buf = fs::read_to_string("../test_dsimple_csvata/nasdaq_data.csv").unwrap();
        let buf = buf.as_bytes();
        let pcd = DataFrame::load_csv(buf, Some(schema));

        assert!(pcd.is_ok());

        let schema = define_schema! {
            "symbol" => DataType::Utf8Str,
            "name" => DataType::Utf8Str,
            "lastsale" => DataType::Float64,
            "marketcap" => DataType::Float64,
            "adr_tso" => DataType::Utf8Str,
            "ipoyear" => DataType::Utf8Str,
            "sector" => DataType::Utf8Str,
            "industry" => DataType::Utf8Str,
            "summary_quote" => DataType::Utf8Str,
            "serialid" => DataType::UInt64,
        };

        let buf = fs::read_to_string("../test_data/nasdaq_data.csv").unwrap();
        let buf = buf.as_bytes();
        let pcd = DataFrame::load_csv(buf, Some(schema));

        assert!(pcd.is_ok());
    }

    #[test]
    fn test_json() {
        let pcd_old = pcd! {
            "column_1" => DataType::Int8: [1i8, 2, 3, 4, 5, 6, 7, 8],
            "column_2" => DataType::Float64: [1.0f64, 2.0, 3.0, 4.0, 22.3, 22.3, 22.3, 22.3],
        };

        let json = pcd_old.to_json();
        let pcd = DataFrame::from_json(&json);

        assert!(pcd.is_ok_and(|inner| inner == pcd_old));
    }
}
