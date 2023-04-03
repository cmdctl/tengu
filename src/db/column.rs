use sqlx::FromRow;

#[derive(Debug, PartialEq, Eq, Hash, Clone, FromRow)]
pub struct Column {
    pub name: String,
    pub table: String,
    pub schema: String,
    pub data_type: String,
    pub is_nullable: bool,
}
