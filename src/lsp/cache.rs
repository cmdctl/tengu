use std::collections::HashSet;
use std::sync::Arc;

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::sql_tools::tools::{get_tables, Table};
use crate::terminal_ui::repository::FsTenguRepository;

pub static ALL_TABLES: Lazy<Arc<Mutex<HashSet<Table>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

pub async fn reset_cache(e: notify::Result<notify::Event>) {
    match e {
        Ok(_) => {
            let mut all_tables = ALL_TABLES.lock().await;
            all_tables.clear();
            let repo = FsTenguRepository::new();
            let tables = get_tables(repo).await.unwrap();
            for table in tables {
                all_tables.insert(table);
            }
        }
        Err(e) => {
            println!("watch error: {:?}", e);
        }
    }
}
