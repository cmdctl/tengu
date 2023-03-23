use anyhow::Result;
use sql_tools::exec::exec_sql;
use terminal_ui::repository::FsTenguRepository;
use tokio::main;

mod terminal_ui;
mod sql_tools;

#[main]
async fn main() -> Result<()> {
    let repo = FsTenguRepository::new();
    let sql = "SELECT TOP 5 * FROM tbl_trade";
    exec_sql(repo, sql).await?;
    Ok(())
}
