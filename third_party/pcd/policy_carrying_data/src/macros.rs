#[macro_export]
macro_rules! push_type {
    ($vec:expr, $data:ident, $ty:tt, $data_type:ident) => {
        // We ignore error for the time being.
        $vec.push_erased(Box::new($data.parse::<$ty>().unwrap_or_default()))
    };
}

#[macro_export]
macro_rules! define_schema {
    ($id:expr, $($name:expr => $ty:path), + $(,)?) => {{
        use $crate::types::*;

        $crate::schema::SchemaBuilder::new()
            $(.add_field_raw($name, $ty, false))*
            .finish_with_executor($id)
    }};

    ($($name:expr => $ty:path), + $(,)?) => {{
        use policy_core::types::*;

        $crate::schema::SchemaBuilder::new()
            $(.add_field_raw($name, $ty, false))*
            .finish()
    }};
}

#[macro_export]
macro_rules! pcd {
  ($($col_name:expr => $ty:path: $content:expr), + $(,)?) => {{
        use $crate::types::*;

        let mut fields = Vec::new();
        let mut field_array = Vec::new();

        $(
            let field = std::sync::Arc::new($crate::field::Field::new($col_name.to_string(), $ty, false, Default::default()));
            let field_data: std::sync::Arc<dyn $crate::field::FieldData> =
                std::sync::Arc::new($crate::field::FieldDataArray::new(field.clone(), $content.to_vec()));
            field_array.push(field_data);
            fields.push(field);
        )*

      $crate::DataFrame::new_with_cols(field_array)
  }};
}
