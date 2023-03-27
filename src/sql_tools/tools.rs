use crate::terminal_ui::repository::TenguRepository;
use anyhow::Result;
use super::exec::get_conn;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Table {
    pub name: String,
    pub schema: String,
}

pub async fn get_tables<R: TenguRepository>(repo: R) -> Result<Vec<Table>> {
    let mut conn = get_conn(repo).await?; 
    let sql = r#"
        SELECT s.name AS schema_name, t.name AS table_name
        FROM sys.tables t
        JOIN sys.schemas s ON t.schema_id = s.schema_id
        ORDER BY s.name, t.name;
    "#;
    let result = conn
        .simple_query(sql)
        .await?
        .into_results()
        .await?
        .into_iter()
        .flatten()
        .map(|row| {
            let schema = row.get::<&str, _>("schema_name").unwrap();
            let name = row.get::<&str, _>("table_name").unwrap();
            Table {
                name: name.to_string(),
                schema: schema.to_string(),
            }
        })
        .collect();
    Ok(result)
}

