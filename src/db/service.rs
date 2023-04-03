use std::collections::HashSet;

use anyhow::Result;

use crate::terminal_ui::repository::FsTenguRepository;

use super::engine::Engine;
use super::mssql::SqlServer;
use super::postgres::Postgres;
use super::{column::Column, table::Table};

#[tower_lsp::async_trait]
pub trait Service {
    async fn get_tables(&self) -> Result<Vec<Table>>;
    async fn get_table_columns(&self, tables: HashSet<Table>) -> Result<HashSet<Column>>;
    fn get_keywords(&self) -> &[&str] {
        &[]
    }
}

#[derive(Debug)]
pub enum TenguService {
    SqlServer(SqlServer<FsTenguRepository>),
    Postgres(Postgres<FsTenguRepository>),
}

impl TenguService {
    pub fn new(engine: Engine, repo: FsTenguRepository) -> Self {
        match engine {
            Engine::SqlServer => {
                let service = SqlServer::new(repo);
                Self::SqlServer(service)
            }
            Engine::Postgres => {
                let service = Postgres::new(repo);
                Self::Postgres(service)
            }
            _ => unimplemented!(),
        }
    }
}

#[tower_lsp::async_trait]
impl Service for TenguService {
    async fn get_tables(&self) -> Result<Vec<Table>> {
        match self {
            Self::SqlServer(service) => service.get_tables().await,
            Self::Postgres(service) => service.get_tables().await,
        }
    }

    async fn get_table_columns(&self, tables: HashSet<Table>) -> Result<HashSet<Column>> {
        match self {
            Self::SqlServer(service) => service.get_table_columns(tables).await,
            Self::Postgres(service) => service.get_table_columns(tables).await,
        }
    }

    fn get_keywords(&self) -> &[&str] {
        match self {
            Self::SqlServer(service) => service.get_keywords(),
            Self::Postgres(service) => service.get_keywords(),
        }
    }
}
