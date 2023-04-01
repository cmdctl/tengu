use tiberius::{
    numeric::Numeric, time::chrono::NaiveDateTime, xml::XmlData, Column, ColumnType, Row, Uuid,
};

pub(crate) fn get_value<'a>(row: &'a Row, col: &Column) -> String {
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
        ColumnType::Int1 | ColumnType::Int2 | ColumnType::Int4 | ColumnType::Intn => {
            match row.try_get::<i32, _>(col.name()) {
                Ok(val) => match val {
                    Some(val) => val.to_string(),
                    None => "NULL".to_string(),
                },
                Err(_) => match row.try_get::<u8, _>(col.name()) {
                    Ok(val) => match val {
                        Some(val) => val.to_string(),
                        None => "NULL".to_string(),
                    },
                    Err(_) => "NULL".to_string(),
                },
            }
        }
        ColumnType::Int8 => {
            if let Some(val) = row.get::<i64, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::BigBinary | ColumnType::BigVarBin => {
            if let Some(val) = row.get::<&'a [u8], _>(col.name()) {
                val.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<String>>()
                    .join(" ")
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Decimaln | ColumnType::Numericn => {
            if let Some(val) = row.get::<Numeric, _>(col.name()) {
                val.to_string()
            } else {
                "NULL".to_string()
            }
        }
        ColumnType::Datetime
        | ColumnType::Timen
        | ColumnType::Datetime2
        | ColumnType::DatetimeOffsetn
        | ColumnType::Datetime4 => {
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
