use anyhow::Result;
use sql_tools::exec::exec_sql;
use terminal_ui::{repository::FsTenguRepository, start_tui};
use tokio::main;
use clap::{Parser, Subcommand};
use std::io;

mod terminal_ui;
mod sql_tools;

#[derive(Subcommand, Debug)]
enum Command {
    Exec,
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
            let mut sql = String::new();
            let mut lines = io::stdin().lines();
            while let Some(line) = lines.next() {
                let line = line?;
                if !line.starts_with("--") {
                    sql.push_str(&line);
                }
            }
            let repo = FsTenguRepository::new();
            exec_sql(repo, &sql).await?;
        }
        None => {
            start_tui()?;
        }
    }
    Ok(())
}
