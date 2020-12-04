use std::collections::HashSet;

use anyhow::{Context, Result};
use sqlx::{mysql::MySqlConnectOptions, MySqlConnection};
use sqlx::{mysql::MySqlSslMode, prelude::*};

const SCHEMA_TABLE: &'static str = "information_schema";

use crate::config::Profile;

pub async fn connect(profile: &Profile) -> Result<MySqlConnection> {
    let conn = MySqlConnectOptions::new()
        .host(&profile.host)
        .port(profile.port)
        .ssl_mode(MySqlSslMode::Disabled);
    let conn = if let Some(ref user) = profile.user {
        conn.username(user)
    } else {
        conn
    };
    let conn = if let Some(ref pass) = profile.password {
        conn.password(pass)
    } else {
        conn
    };
    let conn = if let Some(ref db) = profile.db {
        conn.database(db)
    } else {
        conn
    };
    let conn = if let Some(ref ssl_mode) = profile.ssl_mode {
        let mode = match ssl_mode {
            crate::config::SslMode::Disabled => MySqlSslMode::Disabled,
            crate::config::SslMode::Preferred => MySqlSslMode::Preferred,
            crate::config::SslMode::Required => MySqlSslMode::Required,
            crate::config::SslMode::VerifyCa => MySqlSslMode::VerifyCa,
            crate::config::SslMode::VerifyIdentity => MySqlSslMode::VerifyIdentity,
        };
        conn.ssl_mode(mode)
    } else {
        conn.ssl_mode(MySqlSslMode::Disabled)
    };
    let conn = if let Some(ref ca_file) = profile.ssl_ca {
        conn.ssl_ca(ca_file)
    } else {
        conn
    };
    Ok(conn.connect().await.with_context(|| "连接失败...")?)
}

pub async fn all_databases(conn: &mut MySqlConnection) -> Result<HashSet<String>> {
    let query: Vec<(String,)> = sqlx::query_as("SHOW DATABASES").fetch_all(conn).await?;
    let mut databases = HashSet::new();
    query.into_iter().for_each(|(db,)| {
        databases.insert(db);
    });
    Ok(databases)
}

pub async fn all_tables(conn: &mut MySqlConnection) -> Result<HashSet<String>> {
    let query: Vec<(String,)> = sqlx::query_as("SHOW TABLES").fetch_all(conn).await?;
    let mut tables = HashSet::new();
    query.into_iter().for_each(|(t,)| {
        tables.insert(t);
    });
    Ok(tables)
}

pub async fn scan_database(conn: &mut MySqlConnection) -> Result<Vec<String>> {
    let schema_table = "information_schema";
    let query: Vec<(String,)> = sqlx::query_as("SHOW TABLES")
        .fetch_all(conn)
        .await
        .with_context(|| "无法获取所有table")?;
    let tables: Vec<String> = query.iter().map(|(table,)| table.clone()).collect();
    // let sql = format!(
    //     "SELECT TABLE_NAME,COLUMN_NAME from {} WHERE TABLE_NAME IN ({})",
    //     schema_table,
    //     &tables.join(",")
    // );
    // let colunms: Vec<(String, String)> = sqlx::query_as(&sql).fetch_all(conn).await?;
    Ok(query.into_iter().map(|(table,)| table).collect())
}
