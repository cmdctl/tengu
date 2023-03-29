use super::exec::get_conn;
use crate::terminal_ui::repository::TenguRepository;
use anyhow::Result;
use tiberius::ToSql;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
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

pub async fn get_table_columns<R: TenguRepository>(
    repo: R,
    tables: Vec<Table>,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut conn = get_conn(repo).await?;
    let mut params: Vec<&dyn ToSql> = Vec::new();
    let mut conditions = String::new();

    for (i, table) in tables.iter().enumerate() {
        params.push(&table.schema);
        params.push(&table.name);

        conditions.push_str(&format!(
            "(s.name = @P{} AND t.name = @P{})",
            i * 2 + 1,
            i * 2 + 2,
        ));

        if i < tables.len() - 1 {
            conditions.push_str(" OR ");
        }
    }

    let sql = format!(
        r#"
        SELECT t.name AS table_name, c.name AS column_name
        FROM sys.tables t
        JOIN sys.schemas s ON t.schema_id = s.schema_id
        JOIN sys.columns c ON t.object_id = c.object_id
        WHERE {}
        ORDER BY s.name, t.name, c.column_id;
    "#,
        conditions
    );

    let results = conn
        .query(sql.as_str(), &params)
        .await?
        .into_results()
        .await?
        .into_iter()
        .flatten()
        .map(|row| {
            let table = row.get::<&str, _>("table_name").unwrap();
            let column = row.get::<&str, _>("column_name").unwrap();
            (column.to_string(), table.to_string())
        })
        .collect();

    Ok(results)
}
