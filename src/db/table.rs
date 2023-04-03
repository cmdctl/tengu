use super::column::Column;

use sqlx::FromRow;

#[derive(Debug, PartialEq, Eq, Hash, Clone, FromRow)]
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
