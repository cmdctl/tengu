use super::column::Column;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Table {
    pub name: String,
    pub schema: String,
}

impl From<Column> for Table {
    fn from(column: Column) -> Self {
        Table {
            name: column.table,
            schema: column.schema,
        }
    }
}
