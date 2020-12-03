use anyhow::{Context, Result};
use sqlx::{mysql::MySqlConnectOptions, MySqlConnection};
use sqlx::{mysql::MySqlSslMode, prelude::*};

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
