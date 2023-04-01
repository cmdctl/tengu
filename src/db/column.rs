#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Column {
    pub name: String,
    pub table: String,
    pub schema: String,
    pub data_type: String,
    pub is_nullable: bool,
}
