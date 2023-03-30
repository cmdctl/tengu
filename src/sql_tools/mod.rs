pub mod exec;
pub mod keywords;
pub mod tokenizer;
pub mod tools;

use anyhow::Result;
use std::io;

pub fn sql_from_stdin() -> Result<String> {
    let mut sql = String::new();
    let mut lines = io::stdin().lines();
    while let Some(line) = lines.next() {
        let line = line?;
        if !line.starts_with("--") {
            sql.push_str(&line);
        }
    }
    Ok(sql)
}
