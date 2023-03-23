use tiberius::{Client, Config, AuthMethod, Column, Row, ColumnType, xml::XmlData, time::chrono::NaiveDateTime, Uuid, numeric::Decimal};
use tokio::net::TcpStream;
use tokio_util::compat::{TokioAsyncWriteCompatExt, Compat};
use crate::terminal_ui::repository::TenguRepository;
use anyhow::{Result, anyhow};

pub async fn get_conn<R: TenguRepository>(repo: R) -> Result<Client<Compat<TcpStream>>> {
    let Some(conn) = repo.get_active_connection() else {
        return Err(anyhow!("No active connection found"));
    };
    let mut config = Config::new();
    config.host(conn.host);
    config.port(conn.port.parse::<u16>().unwrap());
    config.database(conn.database);
    config.authentication(AuthMethod::sql_server(&conn.username, &conn.password));
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    Ok(Client::connect(config, tcp.compat_write()).await?)
}

pub async fn exec_sql<R: TenguRepository>(repo: R, sql: &str) -> Result<()> {
    let mut conn = get_conn(repo).await?;
    let stream = conn.simple_query(sql).await?.into_results().await?;
    for rows in stream {
        for (i, row) in rows.iter().enumerate() {
            if i == 0 {
                row.columns().iter().for_each(|col| {
                    print!("{} ", col.name());
                });
                println!();
            }
            row.columns().iter().for_each(|col| {
                println!("{}: {:?}", col.name(), get_value(col, row));
            });
        }
        
    }
    Ok(())
}

fn get_value<'a>(col: &Column, row: &'a Row) -> String {
    match col.column_type() {
        ColumnType::Bit | ColumnType::Bitn => {
            if let Some(val) = row.get::<bool, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Null => "NULL".to_string(),
        ColumnType::Xml => {
            if let Some(val) = row.get::<&'a XmlData, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Guid => {
            if let Some(val) = row.get::<Uuid, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Udt => {
            if let Some(val) = row.get::<&'a str, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Int1 => {
            if let Some(val) = row.get::<u8, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::BigBinary | ColumnType::BigVarBin => {
            if let Some(val) = row.get::<&'a [u8], _>(col.name()) {
                val.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ")
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Int2 => {
            if let Some(val) = row.get::<i16, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Int4 => {
            if let Some(val) = row.get::<i32, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Intn => {
            if let Ok(val) = row.try_get::<u8, _>(col.name()) {
                val.unwrap_or(0).to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Int8 => {
            if let Some(val) = row.get::<i32, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Decimaln | ColumnType::Numericn => {
            if let Some(val) = row.get::<Decimal, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Datetime | ColumnType::Timen | ColumnType::Datetime2 | ColumnType::DatetimeOffsetn | ColumnType::Datetime4 => {
            if let Some(val) = row.get::<NaiveDateTime, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Floatn | ColumnType::Float4 | ColumnType::Float8 => {
            if let Some(val) = row.get::<f64, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        _ => {
            if let Some(val) = row.get::<&'a str, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
    }
}


