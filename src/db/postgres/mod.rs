mod keywords;
use keywords::KEYWORDS;

use anyhow::anyhow;
use anyhow::Result;
use sqlx::postgres::PgConnectOptions;
use sqlx::Connection;
use sqlx::PgConnection;
use std::collections::HashSet;

use crate::terminal_ui::repository::TenguRepository;

use super::column::Column as TenguColumn;
use super::service::Service;
use super::table::Table;

#[derive(Debug)]
pub struct Postgres<T: TenguRepository> {
    repo: T,
}

impl<T: TenguRepository + Sync> Postgres<T> {
    pub fn new(repo: T) -> Self {
        Self { repo }
    }

    pub async fn get_conn(&self) -> Result<PgConnection> {
        let Some(conn) = self.repo.get_active_connection() else {
            return Err(anyhow!("No active connection found"));
        };
        let conn = PgConnection::connect_with(
            &PgConnectOptions::new()
                .database(&conn.database)
                .username(&conn.username)
                .password(&conn.password)
                .host(&conn.host)
                .port(conn.port.parse::<u16>().unwrap()),
        )
        .await?;
        Ok(conn)
    }
}

#[tower_lsp::async_trait]
impl<T: TenguRepository + Sync + Send> Service for Postgres<T> {

    async fn get_tables(&self) -> Result<Vec<Table>> {
        let mut conn = self.get_conn().await?;
        let tables: Vec<Table> = sqlx::query_as::<_, Table>(
            r#"
            SELECT table_schema as schema, table_name as name
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            "#,
        )
        .fetch_all(&mut conn)
        .await?;
        Ok(tables)
    }
    async fn get_table_columns(&self, tables: HashSet<Table>) -> Result<HashSet<TenguColumn>> {
        let sql = format!(
            r#"
            SELECT column_name AS name, table_name AS table, table_schema AS schema, data_type, is_nullable
            FROM information_schema.columns
            WHERE table_name IN ({});
        "#,
            tables
                .iter()
                .map(|t| format!("'{}'", t.name))
                .collect::<Vec<String>>()
                .join(",")
        );
        let mut conn = self.get_conn().await?;
        let columns: Vec<TenguColumn> = sqlx::query_as::<_, TenguColumn>(&sql)
            .fetch_all(&mut conn)
            .await?;
        Ok(columns.into_iter().collect())
    }
    fn get_keywords(&self) -> &[&str] {
        KEYWORDS
    }
}
