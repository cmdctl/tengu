mod exec;
mod keywords;

use crate::db::column::Column as TenguColumn;
use crate::db::table::Table;
use crate::terminal_ui::repository::TenguRepository;
use anyhow::anyhow;
use anyhow::Result;
use prettytable::{Cell, Row as TRow, Table as PrettyTable};
use std::collections::HashSet;
use tiberius::ToSql;
use tiberius::{AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use self::exec::get_value;

use super::service::Service;

#[derive(Debug)]
pub struct SqlServer<T: TenguRepository> {
    repo: T,
}

impl<T: TenguRepository + Sync> SqlServer<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }
    pub async fn get_conn<R: TenguRepository>(&self) -> Result<Client<Compat<TcpStream>>> {
        let Some(conn) = self.repo.get_active_connection() else {
            return Err(anyhow!("No active connection found"));
        };
        let mut config = Config::new();
        config.host(conn.host);
        config.port(conn.port.parse::<u16>().unwrap());
        config.database(conn.database);
        config.authentication(AuthMethod::sql_server(&conn.username, &conn.password));
        config.trust_cert();

        let tcp = TcpStream::connect(config.get_addr()).await?;
        tcp.set_nodelay(true)?;

        Ok(Client::connect(config, tcp.compat_write()).await?)
    }
}

#[tower_lsp::async_trait]
impl<T: TenguRepository + Sync + Send> Service for SqlServer<T> {
    async fn exec_and_print(&self, sql: &str) -> Result<()> {
        let mut conn = self.get_conn::<T>().await?;
        let mut table = PrettyTable::new();
        let stream = conn.simple_query(sql).await?.into_results().await?;
        for rows in stream {
            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    let mut headers = Vec::with_capacity(row.columns().len());
                    row.columns().iter().for_each(|col| {
                        headers.push(Cell::new(col.name()));
                    });
                    table.add_row(TRow::new(headers));
                }
                let mut row_data = Vec::with_capacity(row.columns().len());
                row.columns().iter().for_each(|col| {
                    row_data.push(Cell::new(&get_value(row, col)));
                });
                table.add_row(TRow::new(row_data));
            }
        }
        table.printstd();
        Ok(())
    }
    async fn get_tables(&self) -> Result<Vec<Table>> {
        let mut conn = self.get_conn::<T>().await?;
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
    async fn get_table_columns(&self, tables: HashSet<Table>) -> Result<HashSet<TenguColumn>> {
        let mut conn = self.get_conn::<T>().await?;
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
            SELECT s.name AS schema_name, t.name AS table_name, c.name AS column_name, ic.data_type, ic.is_nullable
            FROM sys.tables t
            JOIN sys.schemas s ON t.schema_id = s.schema_id
            JOIN sys.columns c ON t.object_id = c.object_id
            JOIN information_schema.columns ic ON ic.table_name = t.name AND ic.column_name = c.name
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
                let column = row.get::<&str, _>("column_name").unwrap();
                let table = row.get::<&str, _>("table_name").unwrap();
                let schema = row.get::<&str, _>("schema_name").unwrap();
                let data_type = row.get::<&str, _>("data_type").unwrap();
                let is_nullable = row.get::<&str, _>("is_nullable").unwrap().eq("YES");
                TenguColumn {
                    name: column.to_string(),
                    table: table.to_string(),
                    schema: schema.to_string(),
                    data_type: data_type.to_string(),
                    is_nullable,
                }
            })
            .collect();

        Ok(results)
    }

    fn get_keywords(&self) -> &[&str] {
        return keywords::KEYWORDS;
    }
}
