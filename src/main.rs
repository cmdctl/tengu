use std::collections::HashSet;

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::{service::{TenguService, Service}, table::Table};
use lsp::server::start_lsp;
use terminal_ui::{start_tui, repository::{FsTenguRepository, TenguRepository}};
use tokio::main;

mod db;
mod lsp;
mod prelude;
mod terminal_ui;
mod tokenizer;

#[derive(Subcommand, Debug)]
enum Command {
    Lsp,
    Test,
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    commands: Option<Command>,
}

#[main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.commands {
        Some(Command::Test) => {
            let repo = FsTenguRepository::new();
            let Some(active_connection) = repo.get_active_connection() else {
                eprintln!("No active connection found");
                return Ok(());
            };
            let service = TenguService::new(active_connection.engine, repo);
            let mut set = HashSet::new();
            let table = Table {
                name: "demo".to_string(),
                schema: "public".to_string(),
            };
            set.insert(table.clone());
            let col = service.get_table_columns(set).await?;
            println!("{:?}", col);
        }
        Some(Command::Lsp) => {
            start_lsp().await;
        }
        None => {
            start_tui()?;
        }
    }
    Ok(())
}
