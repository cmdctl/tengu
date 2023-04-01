use anyhow::Result;
use clap::{Parser, Subcommand};
use db::service::{Service, TenguService};
use lsp::server::start_lsp;
use prelude::*;
use terminal_ui::{
    repository::{FsTenguRepository, TenguRepository},
    start_tui,
};
use tokio::main;

mod db;
mod lsp;
mod prelude;
mod terminal_ui;
mod tokenizer;

#[derive(Subcommand, Debug)]
enum Command {
    Exec,
    Lsp,
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
        Some(Command::Exec) => {
            let sql = sql_from_stdin()?;
            let repo = FsTenguRepository::new();
            let Some(active_connection) = repo.get_active_connection() else {
                println!("No active connection");
                return Ok(());
            };
            let service = TenguService::new(active_connection.engine, repo);
            service.exec_and_print(&sql).await?;
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
