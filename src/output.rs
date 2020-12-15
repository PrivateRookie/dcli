use anyhow::anyhow;
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use comfy_table::*;
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use sqlx::{
    mysql::{MySqlColumn, MySqlRow, MySqlValueRef},
    types::time::Date,
    types::time::Time,
    Column, Row, TypeInfo, Value, ValueRef,
};
use std::str::FromStr;

use crate::{config::Config, fl};

#[derive(Debug, Clone, Serialize)]
pub enum Format {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "yaml")]
    Yaml,
    #[serde(rename = "toml")]
    Toml,
    #[serde(rename = "pickle")]
    Pickle,
}

impl Default for Format {
    fn default() -> Self {
        Format::Json
    }
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        if lower == "json" {
            return Ok(Format::Json);
        } else if lower == "yaml" {
            return Ok(Format::Yaml);
        } else if lower == "toml" {
            return Ok(Format::Toml);
        } else if lower == "pickle" {
            return Ok(Format::Pickle);
        } else {
            Err(anyhow!(fl!("invalid-value", val = s)))?
        }
    }
}

pub struct DCliColumn<'a> {
    pub col: &'a MySqlColumn,
    pub val_ref: MySqlValueRef<'a>,
}

pub struct DCliRow(MySqlRow);

pub struct QueryOutput {
    pub rows: Vec<DCliRow>,
}

impl From<Vec<MySqlRow>> for QueryOutput {
    fn from(rows: Vec<MySqlRow>) -> Self {
        let rows = rows.into_iter().map(|r| DCliRow(r)).collect();
        Self { rows }
    }
}

impl Serialize for QueryOutput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.rows.len()))?;
        for row in self.rows.iter() {
            seq.serialize_element(row)?;
        }
        seq.end()
    }
}

impl Serialize for DCliRow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for col in self.0.columns().iter().map(|c| {
            let val_ref = self.0.try_get_raw(c.ordinal()).unwrap();
            DCliColumn { col: c, val_ref }
        }) {
            map.serialize_entry(col.col.name(), &col)?;
        }
        map.end()
    }
}

impl<'a> Serialize for DCliColumn<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let val = ValueRef::to_owned(&self.val_ref);
        if val.is_null() {
            serializer.serialize_none()
        } else {
            match val.type_info().name() {
                "BOOLEAN" => {
                    let v = val.try_decode::<bool>().unwrap();
                    serializer.serialize_bool(v)
                }
                "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED"
                | "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => {
                    let v = val.try_decode::<u64>().unwrap();
                    serializer.serialize_u64(v)
                }
                "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => {
                    let v = val.try_decode::<i64>().unwrap();
                    serializer.serialize_i64(v)
                }
                "FLOAT" => {
                    let v = val.try_decode::<f32>().unwrap();
                    serializer.serialize_f32(v)
                }
                "DOUBLE" => {
                    let v = val.try_decode::<f64>().unwrap();
                    serializer.serialize_f64(v)
                }
                "NULL" => serializer.serialize_none(),
                "DATE" => {
                    let v = val.try_decode::<Date>().unwrap();
                    serializer.serialize_str(&v.to_string())
                }
                "TIME" => {
                    let v = val.try_decode::<Time>().unwrap();
                    serializer.serialize_str(&v.to_string())
                }
                "YEAR" => {
                    let v = val.try_decode::<u64>().unwrap();
                    serializer.serialize_u64(v)
                }
                // TODO add tz config
                "TIMESTAMP" | "DATETIME" => {
                    let v = val.try_decode::<DateTime<Utc>>().unwrap();
                    serializer.serialize_str(&v.to_string())
                }
                "BIT" | "ENUM" | "SET" => {
                    let v = val.try_decode::<String>().unwrap();
                    serializer.serialize_str(&v)
                }
                "DECIMAL" => {
                    let v = val.try_decode::<BigDecimal>().unwrap();
                    serializer.serialize_str(&v.to_string())
                }
                "GEOMETRY" | "JSON" => {
                    let v = val.try_decode::<String>().unwrap();
                    serializer.serialize_str(&v)
                }
                // TODO decode as base64?
                "BINARY" => serializer.serialize_str("BINARY"),
                "VARBINARY" => serializer.serialize_str("VARBINARY"),
                "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                    let v = val.try_decode::<String>().unwrap();
                    serializer.serialize_str(&v)
                }
                "TINYBLOB" => serializer.serialize_str("TINYBLOB"),
                "BLOB" => serializer.serialize_str("BLOB"),
                "MEDIUMBLOB" => serializer.serialize_str("MEDIUMBLOB"),
                "LONGBLOB" => serializer.serialize_str("LONGBLOB"),

                t @ _ => unreachable!(t),
            }
        }
    }
}

impl QueryOutput {
    pub fn to_print_table(&self, config: &Config) -> Table {
        if self.rows.is_empty() {
            return config.new_table();
        }
        let header = self.rows.first().unwrap();
        let header_cols = header.0.columns();
        if header_cols.is_empty() {
            return config.new_table();
        }
        let mut table = config.new_table();
        table.set_header(header_cols.iter().map(|col| col.name()));
        self.rows.iter().for_each(|row| {
            table.add_row(row.0.columns().iter().map(|col| {
                let val_ref = row.0.try_get_raw(col.ordinal()).unwrap();
                let val = ValueRef::to_owned(&val_ref);
                let val = if val.is_null() {
                    Ok(String::new())
                } else {
                    let ty_info = col.type_info();
                    // ref: https://github.com/launchbadge/sqlx/blob/7a707179448a1787f106138f4821ab3fa062db2a/sqlx-core/src/mysql/protocol/text/column.rs#L172
                    match ty_info.name() {
                        "BOOLEAN" => val.try_decode::<bool>().map(|v| v.to_string()),
                        "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED"
                        | "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => {
                            val.try_decode::<u64>().map(|v| v.to_string())
                        }
                        "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => {
                            val.try_decode::<i64>().map(|v| v.to_string())
                        }
                        "FLOAT" => val.try_decode::<f32>().map(|v| v.to_string()),
                        "DOUBLE" => val.try_decode::<f64>().map(|v| v.to_string()),
                        "NULL" => Ok("NULL".to_string()),
                        "DATE" => val.try_decode::<Date>().map(|v| v.to_string()),
                        "TIME" => val.try_decode::<Time>().map(|v| v.to_string()),
                        "YEAR" => val.try_decode::<u64>().map(|v| v.to_string()),
                        // TODO add tz config
                        "TIMESTAMP" | "DATETIME" => {
                            val.try_decode::<DateTime<Utc>>().map(|v| v.to_string())
                        }
                        "BIT" | "ENUM" | "SET" => val.try_decode::<String>(),
                        "DECIMAL" => val.try_decode::<BigDecimal>().map(|v| v.to_string()),
                        "GEOMETRY" | "JSON" => val.try_decode::<String>(),
                        "BINARY" => Ok("BINARY".to_string()),
                        "VARBINARY" => Ok("VARBINARY".to_string()),
                        "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                            val.try_decode::<String>()
                        }
                        "TINYBLOB" => Ok("TINYBLOB".to_string()),
                        "BLOB" => Ok("BLOB".to_string()),
                        "MEDIUMBLOB" => Ok("MEDIUMBLOB".to_string()),
                        "LONGBLOB" => Ok("LONGBLOB".to_string()),

                        t @ _ => unreachable!(t),
                    }
                };
                val.unwrap()
            }));
        });
        table
    }
}
