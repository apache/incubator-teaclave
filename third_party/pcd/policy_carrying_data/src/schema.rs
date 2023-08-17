use std::sync::Arc;

use hashbrown::HashMap;
use policy_core::types::DataType;
use serde::{Deserialize, Serialize};

use crate::field::{new_empty, Field, FieldData, FieldRef};

pub type SchemaRef = Arc<Schema>;
pub type SchemaMetadata = HashMap<String, String>;

/// A builder that avoids manually constructing a new [`Schema`].
#[derive(Clone, Debug, Default)]
pub struct SchemaBuilder {
    fields: Vec<FieldRef>,
}

impl From<SchemaRef> for SchemaBuilder {
    fn from(value: SchemaRef) -> Self {
        Self {
            fields: value.fields.clone(),
        }
    }
}

impl SchemaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a [`FieldRef`] into schema.
    pub fn add_field(mut self, field: FieldRef) -> Self {
        // Check if there is name collision.
        let name_repeat = self
            .fields
            .iter_mut()
            .find(|this_field| this_field.name == field.name);

        match name_repeat {
            Some(this_field) => {
                // Not the 'same' trait.
                if !Arc::ptr_eq(this_field, &field) {
                    // Replace the underlying field with the new one.
                    match Arc::get_mut(this_field) {
                        Some(old) => {
                            // Move to `_` and drop it when out of scope.
                            let _ = std::mem::replace(old, field.as_ref().clone());
                        }
                        None => {
                            // Failed to mutate the inner value. We just let the Arc point to field.
                            *this_field = Arc::new(field.as_ref().clone());
                        }
                    }
                }
            }
            None => self.fields.push(field),
        }

        self
    }

    pub fn add_field_raw(self, name: &str, data_type: DataType, nullable: bool) -> Self {
        let field = Arc::new(Field {
            name: name.into(),
            data_type,
            nullable,
            metadata: Default::default(),
        });

        self.add_field(field)
    }

    #[inline]
    pub fn finish(self) -> Arc<Schema> {
        Arc::new(Schema {
            fields: self.fields,
            metadata: Default::default(),
        })
    }
}

/// This struct represents a schema of the input data which, in most cases, is in a table form.
/// Schema for such data types, in fact, is something that describes the attribute/column of the table.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    /// The fields of the table.
    pub(crate) fields: Vec<FieldRef>,
    /// The matadata of the schema.
    #[allow(unused)]
    pub(crate) metadata: SchemaMetadata,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            metadata: Default::default(),
            fields: Vec::new(),
        }
    }
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl Schema {
    /// Constructs a new schema from an array of field descriptions.
    pub fn new(fields: Vec<FieldRef>, metadata: SchemaMetadata) -> Self {
        Self { fields, metadata }
    }

    /// Merges two different schemas.
    pub fn merge(&mut self, other: Self) {
        self.fields.extend(other.fields.into_iter())
    }

    /// Gets the column as owned.
    #[inline]
    pub fn fields_owned(&self) -> Vec<FieldRef> {
        self.fields.iter().cloned().collect()
    }

    /// Gets the column as a reference.
    #[inline]
    pub fn fields(&self) -> &[Arc<Field>] {
        self.fields.as_ref()
    }

    #[inline]
    /// Gets empty data columns.
    pub fn empty_field_data(&self) -> Vec<Box<dyn FieldData>> {
        self.fields
            .iter()
            .map(|column| new_empty(column.clone()))
            .collect()
    }
}
