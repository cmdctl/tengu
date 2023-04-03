use anyhow::Result;
use clap::{Parser, Subcommand};
use lsp::server::start_lsp;
use terminal_ui::start_tui;
use tokio::main;

mod db;
mod lsp;
mod prelude;
mod terminal_ui;
mod tokenizer;

#[derive(Subcommand, Debug)]
enum Command {
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
        Some(Command::Lsp) => {
            start_lsp().await;
        }
        None => {
            start_tui()?;
        }
    }
    Ok(())
}
