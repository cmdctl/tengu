use anyhow::Result;
use clap::{Parser, Subcommand};
use lsp::server::start_lsp;
use sql_tools::exec::exec_sql;
use sql_tools::sql_from_stdin;
use terminal_ui::{repository::FsTenguRepository, start_tui};
use tokio::main;

mod lsp;
mod prelude;
mod sql_tools;
mod terminal_ui;

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
            exec_sql(repo, &sql).await?;
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
