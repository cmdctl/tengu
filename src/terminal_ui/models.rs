use std::fmt::Display;

use super::repository::{FsTenguRepository, TenguRepository};
use serde::{Deserialize, Serialize};
use tui::widgets::ListState;

pub enum InputMode {
    Normal,
    Name,
    Engine,
    Host,
    Port,
    Username,
    Password,
    Database,
    Submit,
    Search,
    List,
    Delete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Engine {
    #[serde(rename = "sqlserver")]
    SqlServer,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "mysql")]
    Mysql,
}

impl Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Engine::SqlServer => write!(f, "sqlserver"),
            Engine::Postgres => write!(f, "postgres"),
            Engine::Mysql => write!(f, "mysql"),
        }
    }
}

impl From<String> for Engine {
    fn from(engine: String) -> Self {
        match engine.as_str() {
            "sqlserver" => Engine::SqlServer,
            "postgres" => Engine::Postgres,
            "mysql" => Engine::Mysql,
            _ => Engine::SqlServer,
        }
    }
}

impl From<Engine> for String {
    fn from(engine: Engine) -> Self {
        match engine {
            Engine::SqlServer => "sqlserver".to_string(),
            Engine::Postgres => "postgres".to_string(),
            Engine::Mysql => "mysql".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub name: String,
    pub engine: Engine,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

impl Connection {
    pub fn new(
        name: String,
        engine: Engine,
        host: String,
        port: String,
        username: String,
        password: String,
        database: String,
    ) -> Connection {
        Connection {
            name,
            engine,
            host,
            port,
            username,
            password,
            database,
        }
    }
}

pub struct Tengu<R: TenguRepository> {
    pub repo: R,
    pub mode: InputMode,
    pub list_state: ListState,
    pub engines_list_state: ListState,
    pub engines: Vec<Engine>,
    pub connections: Vec<Connection>,
    pub search_txt: String,
    pub search_list: Vec<Connection>,
    pub new_name: String,
    pub new_engine: Option<Engine>,
    pub new_host: String,
    pub new_port: String,
    pub new_username: String,
    pub new_password: String,
    pub new_database: String,
    pub edit_mode: bool,
    pub edit_index: Option<usize>,
    pub active_connection: Option<Connection>,
}

impl Tengu<FsTenguRepository> {
    pub fn new() -> Tengu<FsTenguRepository> {
        let repo = FsTenguRepository::new();
        let connections = repo.list();
        Tengu {
            repo,
            mode: InputMode::List,
            list_state: ListState::default(),
            engines_list_state: ListState::default(),
            engines: vec![Engine::SqlServer, Engine::Postgres, Engine::Mysql],
            connections,
            search_txt: String::new(),
            search_list: Vec::new(),
            new_name: String::new(),
            new_engine: None,
            new_host: String::new(),
            new_port: String::new(),
            new_username: String::new(),
            new_password: String::new(),
            new_database: String::new(),
            edit_mode: false,
            edit_index: None,
            active_connection: None,
        }
    }
    pub fn change_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }
    pub fn clear_fields(&mut self) {
        self.engines_list_state.select(None);

        self.new_name.clear();
        self.new_engine = None;
        self.new_host.clear();
        self.new_port.clear();
        self.new_username.clear();
        self.new_password.clear();
        self.new_database.clear();
    }

    pub fn insert(&mut self) {
        let connection = Connection::new(
            self.new_name.clone(),
            self.new_engine.clone().unwrap(),
            self.new_host.clone(),
            self.new_port.clone(),
            self.new_username.clone(),
            self.new_password.clone(),
            self.new_database.clone(),
        );
        self.repo.insert(&connection);
        self.connections.push(connection);
        self.clear_fields();
        self.change_mode(InputMode::Normal);
    }
    pub fn start_edit_mode(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let connection = &self.connections[index];
            self.new_name = connection.name.to_owned();
            self.new_engine = connection.engine.clone().into();
            self.new_host = connection.host.to_owned();
            self.new_port = connection.port.to_owned();
            self.new_username = connection.username.to_owned();
            self.new_password = connection.password.to_owned();
            self.new_database = connection.database.to_owned();
            self.edit_mode = true;
            self.edit_index = Some(index);
            self.change_mode(InputMode::Name);
        }
    }
    pub fn edit(&mut self) {
        let index = self.edit_index.unwrap();
        let connection = Connection::new(
            self.new_name.to_owned(),
            self.new_engine.clone().unwrap(),
            self.new_host.to_owned(),
            self.new_port.to_owned(),
            self.new_username.to_owned(),
            self.new_password.to_owned(),
            self.new_database.to_owned(),
        );
        self.repo.update(&connection);
        self.connections[index] = connection;
        self.clear_fields();
        self.end_edit_mode();
        self.change_mode(InputMode::List);
    }
    pub fn end_edit_mode(&mut self) {
        if self.edit_mode {
            self.edit_mode = false;
            self.edit_index = None;
        }
    }
    pub fn check_delete(&mut self) {
        if self.list_state.selected().is_some() {
            self.change_mode(InputMode::Delete);
        }
    }
    pub fn delete(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let name = self.connections[index].name.to_owned();
            self.connections.remove(index);
            self.repo.delete(name);
            if index > 0 {
                self.list_state.select(Some(0));
            } else {
                self.list_state.select(None);
            }
            self.engines_list_state.select(None);
            self.change_mode(InputMode::List);
        }
    }
    pub fn search(&mut self) {
        self.search_list = self
            .connections
            .clone()
            .into_iter()
            .filter(|item| item.name.starts_with(&self.search_txt.to_owned()))
            .collect();
    }
    pub fn move_up(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == 0 {
                    Some(v)
                } else {
                    Some(v - 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }
    pub fn move_down(&mut self) {
        let selected = match self.list_state.selected() {
            Some(v) => {
                if v == self.connections.len() - 1 {
                    Some(v)
                } else {
                    Some(v + 1)
                }
            }
            None => Some(0),
        };
        self.list_state.select(selected);
    }

    pub fn engine_move_up(&mut self) {
        let selected = match self.engines_list_state.selected() {
            Some(v) => {
                if v == 0 {
                    Some(v)
                } else {
                    Some(v - 1)
                }
            }
            None => Some(0),
        };
        self.engines_list_state.select(selected);
    }

    pub fn engine_move_down(&mut self) {
        let selected = match self.engines_list_state.selected() {
            Some(v) => {
                if v == self.engines.len() - 1 {
                    Some(v)
                } else {
                    Some(v + 1)
                }
            }
            None => Some(0),
        };
        self.engines_list_state.select(selected);
    }

    pub fn pick_engine(&mut self) {
        if let Some(index) = self.engines_list_state.selected() {
            self.new_engine = Some(self.engines[index].clone());
        }
    }
    pub fn activate_connection(&mut self) {
        if let Some(index) = self.list_state.selected() {
            let connection = self.connections[index].clone();
            self.repo.activate_connection(&connection);
            self.active_connection = Some(connection);
        }
    }
}
