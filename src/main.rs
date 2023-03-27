use anyhow::Result;
use sql_tools::{sql_from_stdin, tools::get_tables};
use sql_tools::exec::exec_sql;
use terminal_ui::{repository::FsTenguRepository, start_tui};
use tokio::main;
use clap::{Parser, Subcommand};
use lsp::server::start_lsp;

mod terminal_ui;
mod sql_tools;
mod lsp;

#[derive(Subcommand, Debug)]
enum Command {
    Exec,
    Lsp,
    Tables,
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
        Some(Command::Tables) => {
            let repo = FsTenguRepository::new();
            let tables = get_tables(repo).await?;
            for table in tables {
                println!("{:?}", table);
            }
        }
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
