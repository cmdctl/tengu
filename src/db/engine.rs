use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
