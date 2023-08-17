use tabled::{
    builder::Builder,
    settings::{object::Rows, Alignment, Modify, Style},
};

use crate::row::RowSet;

/// Print the rows as a pretty table.
pub fn print_rows(rows: &RowSet) -> String {
    let row_num = rows.rows.len();

    let content = rows
        .schema
        .iter()
        .map(|field| field.to_string())
        .collect::<Vec<_>>();
    let rows = rows
        .rows
        .iter()
        .map(|row| row.stringify())
        .collect::<Vec<_>>();

    let mut builder = Builder::new();
    builder.push_record(content);
    for row in rows {
        builder.push_record(row);
    }

    let mut table = builder.build();
    table
        .with(Style::ascii())
        .with(Modify::new(Rows::new(1..)).with(Alignment::left()));
    format!("{}\n{} rows in set", table.to_string(), row_num)
}
