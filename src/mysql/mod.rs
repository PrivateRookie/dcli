use std::collections::{HashMap, HashSet};

use crate::{config::Profile, output::QueryOutput};
use anyhow::{Context, Result};
use chrono::FixedOffset;
use sqlx::{
    mysql::{MySqlConnectOptions, MySqlRow, MySqlSslMode},
    MySqlPool,
};

mod constants;
pub use constants::{KEYWORDS, SCHEMA_TABLE};

/// stand for mysql client server session, containing tz info etc...
pub struct Session {
    pool: MySqlPool,
}

impl Session {
    /// create session with profile
    pub async fn connect_with(profile: &Profile) -> Result<Self> {
        let options = MySqlConnectOptions::new()
            .host(&profile.host)
            .port(profile.port)
            .ssl_mode(MySqlSslMode::Disabled);
        let options = if let Some(ref user) = profile.user {
            options.username(user)
        } else {
            options
        };
        let options = if let Some(ref pass) = profile.password {
            options.password(pass)
        } else {
            options
        };
        let options = options.database(&profile.db);
        let options = if let Some(ref ssl_mode) = profile.ssl_mode {
            let mode = match ssl_mode {
                crate::config::SslMode::Disabled => MySqlSslMode::Disabled,
                crate::config::SslMode::Preferred => MySqlSslMode::Preferred,
                crate::config::SslMode::Required => MySqlSslMode::Required,
                crate::config::SslMode::VerifyCa => MySqlSslMode::VerifyCa,
                crate::config::SslMode::VerifyIdentity => MySqlSslMode::VerifyIdentity,
            };
            options.ssl_mode(mode)
        } else {
            options.ssl_mode(MySqlSslMode::Disabled)
        };
        let options = if let Some(ref ca_file) = profile.ssl_ca {
            options.ssl_ca(ca_file)
        } else {
            options
        };
        let pool = MySqlPool::connect_with(options)
            .await
            .with_context(|| crate::fl!("connect-failed"))?;
        Ok(Self { pool })
    }

    pub async fn all_databases(&self) -> Result<HashSet<String>> {
        let query: Vec<(String,)> = sqlx::query_as("SHOW DATABASES")
            .fetch_all(&self.pool)
            .await?;
        let mut databases = HashSet::new();
        query.into_iter().for_each(|(db,)| {
            databases.insert(db);
        });
        Ok(databases)
    }

    pub async fn all_tables(&self) -> Result<HashSet<String>> {
        let query: Vec<(String,)> = sqlx::query_as("SHOW TABLES").fetch_all(&self.pool).await?;
        let mut tables = HashSet::new();
        query.into_iter().for_each(|(t,)| {
            tables.insert(t);
        });
        Ok(tables)
    }

    pub async fn all_columns(
        &self,
        tables: &HashSet<String>,
    ) -> Result<HashMap<String, HashSet<String>>> {
        let mut columns: HashMap<String, HashSet<String>> = HashMap::new();
        if tables.is_empty() {
            return Ok(columns);
        }

        let sql = format!(
            "SELECT TABLE_NAME, COLUMN_NAME FROM {}.COLUMNS WHERE table_name IN ({})",
            SCHEMA_TABLE,
            tables
                .iter()
                .map(|t| format!("'{}'", t))
                .collect::<Vec<String>>()
                .join(",")
        );
        let query: Vec<(String, String)> = sqlx::query_as(&sql).fetch_all(&self.pool).await?;
        query.into_iter().for_each(|(table, col)| {
            if let Some(table) = columns.get_mut(&table) {
                table.insert(col);
            } else {
                let mut entry = HashSet::new();
                entry.insert(col);
                columns.insert(table, entry);
            }
        });
        Ok(columns)
    }

    pub async fn tz_offset(&self) -> Result<FixedOffset> {
        let (offset,): (i32,) =
            sqlx::query_as("SELECT TIME_TO_SEC(TIMEDIFF(NOW(), UTC_TIMESTAMP));")
                .fetch_one(&self.pool)
                .await
                .with_context(|| "tz fetch error")?;
        Ok(FixedOffset::east(offset))
    }

    pub async fn query(&self, to_exec: &str) -> Result<QueryOutput> {
        let rows: Vec<MySqlRow> = sqlx::query(to_exec)
            .fetch_all(&self.pool)
            .await
            .with_context(|| "")?;
        Ok(QueryOutput { rows })
    }

    pub async fn close(&self) {
        self.pool.close().await
    }
}
