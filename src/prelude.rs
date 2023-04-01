use anyhow::Result;
use std::collections::HashSet;
use std::io;
use std::{
    fs::File,
    hash::Hash,
    io::{BufReader, Read},
    path::PathBuf,
};

pub trait Concat<T> {
    fn concat(&mut self, other: &Option<Vec<T>>) -> Self;
}

impl<T: Clone> Concat<T> for Option<Vec<T>> {
    fn concat(&mut self, other: &Option<Vec<T>>) -> Self {
        if let Some(other_vec) = other {
            match self {
                Some(vec) => vec.extend(other_vec.iter().cloned()),
                None => *self = Some(other_vec.clone()),
            }
        }
        self.clone()
    }
}

pub trait SetEq<T>
where
    T: Eq + Hash,
{
    fn equals(&self, other: &HashSet<T>) -> bool;
}

impl<T> SetEq<T> for HashSet<T>
where
    T: Eq + Hash,
{
    fn equals(&self, other: &HashSet<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for item in self.iter() {
            if !other.contains(item) {
                return false;
            }
        }
        true
    }
}

pub fn read_file_to_string(file_path: PathBuf) -> std::io::Result<String> {
    let file = File::open(file_path)?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::with_capacity(size as usize);
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

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
