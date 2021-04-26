use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Serialize,
};
use sqlx::{
    mysql::{MySqlColumn, MySqlRow, MySqlValueRef},
    types::time::{Date, Time},
    Column, Row, TypeInfo, Value, ValueRef,
};
use std::{str::FromStr, vec};

use crate::{config::Config, fl};

#[derive(Debug, Clone, Serialize)]
pub enum Format {
    #[serde(rename = "csv")]
    Csv,
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
            Ok(Format::Json)
        } else if lower == "csv" {
            Ok(Format::Csv)
        } else if lower == "yaml" {
            Ok(Format::Yaml)
        } else if lower == "toml" {
            Ok(Format::Toml)
        } else if lower == "pickle" {
            Ok(Format::Pickle)
        } else {
            Err(anyhow!(fl!("invalid-value", val = s)))
        }
    }
}
pub struct QueryOutput {
    pub rows: Vec<MySqlRow>,
}
pub struct DCliColumn<'a> {
    pub col: &'a MySqlColumn,
    pub val_ref: MySqlValueRef<'a>,
}

pub struct QueryOutputMapSer<'a>(pub &'a QueryOutput);
struct DcliRowMapSer<'a>(&'a MySqlRow);
struct QueryOutputListSer<'a>(&'a QueryOutput);
struct DcliRowListSer<'a>(&'a MySqlRow);

impl<'a> Serialize for QueryOutputMapSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.rows.len()))?;
        for row in self.0.rows.iter().map(DcliRowMapSer) {
            seq.serialize_element(&row)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for DcliRowMapSer<'a> {
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

impl<'a> Serialize for QueryOutputListSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.rows.len()))?;
        for row in self.0.rows.iter().map(DcliRowListSer) {
            seq.serialize_element(&row)?;
        }
        seq.end()
    }
}

impl<'a> Serialize for DcliRowListSer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for col in self.0.columns().iter().map(|c| {
            let val_ref = self.0.try_get_raw(c.ordinal()).unwrap();
            DCliColumn { col: c, val_ref }
        }) {
            seq.serialize_element(&col)?;
        }
        seq.end()
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
                // NOTE not sure for this
                // ref https://dev.mysql.com/doc/refman/8.0/en/time-zone-support.html
                "DATETIME" => {
                    let v = val
                        .try_decode::<sqlx::types::time::OffsetDateTime>()
                        .unwrap();
                    serializer.serialize_str(&v.to_string())
                }
                "TIMESTAMP" => {
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
                "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                    let v = val.try_decode::<String>().unwrap();
                    serializer.serialize_str(&v)
                }
                "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => {
                    let b64_str = val.try_decode::<Vec<u8>>().map(base64::encode).unwrap();
                    serializer.serialize_str(&b64_str)
                }
                t => unreachable!(t),
            }
        }
    }
}

impl QueryOutput {
    fn convert_col(row: &MySqlRow, col: &MySqlColumn) -> String {
        let val_ref = row.try_get_raw(col.ordinal()).unwrap();
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
                // NOTE not sure for this
                "DATETIME" => val
                    .try_decode::<sqlx::types::time::OffsetDateTime>()
                    .map(|v| v.to_string()),
                "TIMESTAMP" => val
                    .try_decode::<chrono::DateTime<Utc>>()
                    .map(|v| v.to_string()),
                "BIT" | "ENUM" | "SET" => val.try_decode::<String>(),
                "DECIMAL" => val.try_decode::<BigDecimal>().map(|v| v.to_string()),
                "GEOMETRY" | "JSON" => val.try_decode::<String>(),
                "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                    val.try_decode::<String>()
                }
                "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => {
                    val.try_decode::<Vec<u8>>().map(base64::encode)
                }
                t => unreachable!(t),
            }
        };
        val.unwrap()
    }

    fn vertical_print_table(&self, config: &Config) {
        if self.rows.is_empty() {
            return;
        }
        let keys = self.rows.first().unwrap().columns();
        if keys.is_empty() {
            return;
        }
        for (row_idx, row) in self.rows.iter().enumerate() {
            let mut table = config.new_table();
            table.load_preset("        :          ");

            for (idx, col) in row.columns().iter().enumerate() {
                table.add_row(&[
                    keys.get(idx).unwrap().name(),
                    &QueryOutput::convert_col(row, col),
                ]);
            }
            println!(
                "[{:4} row] *******************************************",
                row_idx
            );
            println!("{}", table);
        }
    }

    fn horizontal_print_table(&self, config: &Config) {
        if self.rows.is_empty() {
            return;
        }
        let header = self.rows.first().unwrap();
        let header_cols = header.columns();
        if header_cols.is_empty() {
            return;
        }
        let mut table = config.new_table();
        table.set_header(header_cols.iter().map(|col| col.name()));
        self.rows.iter().for_each(|row| {
            table.add_row(
                row.columns()
                    .iter()
                    .map(|col| QueryOutput::convert_col(row, col)),
            );
        });
        println!("{}", table)
    }

    pub fn to_print_table(&self, config: &Config, vertical: bool) {
        if vertical {
            self.vertical_print_table(config);
        } else {
            self.horizontal_print_table(config);
        }
    }

    pub fn to_csv(&self) -> Result<String> {
        if self.rows.is_empty() {
            return Ok(String::new());
        }
        let headers = self
            .rows
            .first()
            .unwrap()
            .columns()
            .iter()
            .map(|c| c.name())
            .collect::<Vec<&str>>()
            .join(",");
        let mut out = vec![];
        out.extend(headers.as_bytes());
        out.push(b'\n');
        {
            let mut wtr = csv::Writer::from_writer(&mut out);
            for row in self.rows.iter().map(DcliRowListSer) {
                wtr.serialize(row)
                    .with_context(|| fl!("serialize-output-failed"))?;
            }
        }
        Ok(String::from_utf8_lossy(&out).to_string())
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&QueryOutputMapSer(self))
            .with_context(|| fl!("serialize-output-failed"))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(&QueryOutputMapSer(self))
            .with_context(|| fl!("serialize-output-failed"))
    }

    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(&QueryOutputMapSer(self))
            .with_context(|| fl!("serialize-output-failed"))
    }

    pub fn to_pickle(&self) -> Result<Vec<u8>> {
        let mut out = vec![];
        serde_pickle::to_writer(&mut out, &QueryOutputMapSer(self), false)
            .with_context(|| fl!("serialize-output-failed"))?;
        Ok(out)
    }
}
