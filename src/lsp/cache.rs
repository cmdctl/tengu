use std::collections::HashSet;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use tower_lsp::lsp_types::CompletionItem;

use crate::sql_tools::tools::{get_tables, Table};
use crate::terminal_ui::repository::FsTenguRepository;

pub static ALL_TABLES: Lazy<Arc<Mutex<HashSet<Table>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

pub static TABLES_IN_FILE: Lazy<Arc<Mutex<HashSet<Table>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

pub static ALL_COLUMNS: Lazy<Arc<Mutex<Vec<CompletionItem>>>> =
    Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

pub async fn reset_cache(e: notify::Result<notify::Event>) {
    match e {
        Ok(_) => {
            let mut all_tables = ALL_TABLES.lock().await;
            all_tables.clear();
            let mut tables_in_file = TABLES_IN_FILE.lock().await;
            tables_in_file.clear();
            let mut all_columns = ALL_COLUMNS.lock().await;
            all_columns.clear();
            let repo = FsTenguRepository::new();
            if let Ok(tables) = get_tables(repo).await {
                for table in tables {
                    all_tables.insert(table);
                }
            }
        }
        Err(e) => {
            println!("watch error: {:?}", e);
        }
    }
}
