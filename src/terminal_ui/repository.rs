use super::models::Connection;
use std::{fs, io::Write, path::PathBuf};

#[derive(Debug, Clone)]
pub struct FsTenguRepository {
    base_path: PathBuf,
    active_conn_file_path: PathBuf,
}

impl FsTenguRepository {
    pub(crate) fn new() -> FsTenguRepository {
        let base_path = dirs::home_dir().unwrap().join(".config").join("tengu");
        let active_conn_file_path = base_path.join(".active");
        fs::create_dir_all(&base_path).unwrap();
        if !active_conn_file_path.exists() {
            fs::File::create(&active_conn_file_path).unwrap();
        }
        FsTenguRepository {
            base_path,
            active_conn_file_path,
        }
    }
}

pub trait TenguRepository {
    fn insert(&self, connection: &Connection);
    fn update(&self, connection: &Connection);
    fn delete(&self, name: String);
    fn activate_connection(&mut self, connection: &Connection);
    fn active_connection_path(&self) -> PathBuf;
    fn get_active_connection(&self) -> Option<Connection>;
    fn list(&self) -> Vec<Connection>;
}

impl TenguRepository for FsTenguRepository {
    fn insert(&self, connection: &Connection) {
        let conn_path = self.base_path.join(&connection.name).with_extension("json");
        let mut conn_file = fs::File::create(conn_path).unwrap();
        let conn_json = serde_json::to_string_pretty(connection).unwrap();
        conn_file.write_all(conn_json.as_bytes()).unwrap();
    }
    fn update(&self, connection: &Connection) {
        let conn_path = self.base_path.join(&connection.name).with_extension("json");
        let mut conn_file = fs::File::create(conn_path).unwrap();
        let conn_json = serde_json::to_string_pretty(connection).unwrap();
        conn_file.write_all(conn_json.as_bytes()).unwrap();
    }
    fn delete(&self, name: String) {
        let conn_path = self.base_path.join(&name).with_extension("json");
        fs::remove_file(conn_path).unwrap();
    }
    fn activate_connection(&mut self, connection: &Connection) {
        let mut active_conn = fs::File::create(self.active_conn_file_path.clone()).unwrap();
        let conn_json = serde_json::to_string_pretty(connection).unwrap();
        active_conn.write_all(conn_json.as_bytes()).unwrap();
    }

    fn get_active_connection(&self) -> Option<Connection> {
        let Ok(active_conn_file) = fs::File::open(self.active_conn_file_path.clone()) else {
            println!("No active connection found");
            return None;
        };
        if let Ok(conn) = serde_json::from_reader(active_conn_file) {
            Some(conn)
        } else {
            None
        }
    }

    fn list(&self) -> Vec<Connection> {
        let mut connections = Vec::new();
        let files = fs::read_dir(self.base_path.clone()).unwrap();
        files.into_iter().map(|f| f.unwrap()).for_each(|f| {
            let conn_path = f.path();
            if let Some(ext) = conn_path.extension() {
                if ext != "json" {
                    return;
                }
                let conn_file = fs::File::open(conn_path).unwrap();
                match serde_json::from_reader(conn_file) {
                    Ok(conn) => connections.push(conn),
                    Err(e) => println!("Error reading connection file: {}", e),
                }
            }
        });
        connections
    }

    fn active_connection_path(&self) -> PathBuf {
        let base_path = dirs::home_dir().unwrap().join(".config").join("tengu");
        let active_conn_file_path = base_path.join(".active");
        return active_conn_file_path;
    }
}
