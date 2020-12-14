use crate::{
    config::{Config, Profile, SslMode, TableStyle},
    mysql::connect,
    utils::read_file,
};
use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use comfy_table::*;
use crate::fl;
use sqlx::{
    mysql::MySqlRow, types::time::Date, types::time::Time, Column, Row, TypeInfo, Value, ValueRef,
};
use structopt::StructOpt;

pub mod shell;

#[cfg_attr(feature = "zh-CN", doc = "数据库连接工具")]
#[cfg_attr(feature = "en-US", doc = "database connection manage")]
#[derive(Debug, StructOpt)]
#[structopt(name = "dcli")]
pub enum DCliCommand {
    #[cfg_attr(feature = "zh-CN", doc = "配置相关命令 profile 命令别名")]
    #[cfg_attr(feature = "en-US", doc = "profile commands alias")]
    P {
        #[structopt(subcommand)]
        cmd: ProfileCmd,
    },
    #[cfg_attr(feature = "zh-CN", doc = "配置相关命令")]
    #[cfg_attr(feature = "en-US", doc = "profile command")]
    Profile {
        #[structopt(subcommand)]
        cmd: ProfileCmd,
    },
    #[cfg_attr(feature = "zh-CN", doc = "显示样式相关命令")]
    #[cfg_attr(feature = "en-US", doc = "style commands")]
    Style {
        #[structopt(subcommand)]
        cmd: StyleCmd,
    },
    #[cfg_attr(feature = "zh-CN", doc = "使用 `mysql` 命令连接到 mysql")]
    #[cfg_attr(feature = "en-US", doc = "use `mysql` connect to mysql server")]
    Conn {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        profile: String,
    },
    #[cfg_attr(feature = "zh-CN", doc = "使用一个配置运行命令")]
    #[cfg_attr(feature = "en-US", doc = "use a profile to exec sql")]
    Exec {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        #[structopt(short, long)]
        profile: String,

        #[cfg_attr(
            feature = "zh-CN",
            doc = "命令 使用 @<文件路径> 读取 SQL 文件内容作为输入"
        )]
        #[cfg_attr(
            feature = "en-US",
            doc = "sql, use @<file_path> to read SQL file as input"
        )]
        command: Vec<String>,
    },

    #[cfg_attr(feature = "zh-CN", doc = "运行连接到 mysql 的 shell")]
    #[cfg_attr(feature = "en-US", doc = "launch shell connected to mysql")]
    Shell {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        #[structopt(short, long)]
        profile: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum ProfileCmd {
    #[cfg_attr(feature = "zh-CN", doc = "列出所有配置")]
    #[cfg_attr(feature = "en-US", doc = "list all")]
    List,
    #[cfg_attr(feature = "zh-CN", doc = "添加一个配置")]
    #[cfg_attr(feature = "en-US", doc = "add a profile")]
    Add {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        name: String,

        #[cfg_attr(feature = "zh-CN", doc = "是否强制覆盖")]
        #[cfg_attr(feature = "en-US", doc = "force override")]
        #[structopt(short, long)]
        force: bool,

        #[structopt(flatten)]
        profile: Profile,
    },
    #[cfg_attr(feature = "zh-CN", doc = "删除一个配置")]
    #[cfg_attr(feature = "en-US", doc = "delete a profile")]
    Del {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        profile: String,
    },
    #[cfg_attr(feature = "zh-CN", doc = "修改已有配置")]
    #[cfg_attr(feature = "en-US", doc = "edit a profile")]
    Set {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        name: String,

        #[cfg_attr(feature = "zh-CN", doc = "数据库 hostname, IPv6地址请使用'[]'包围")]
        #[cfg_attr(
            feature = "en-US",
            doc = "database hostname, IPv6 should be surrounded by '[]'"
        )]
        #[structopt(short = "h", long)]
        host: Option<String>,

        #[cfg_attr(feature = "zh-CN", doc = "数据库 port 1 ~ 65535")]
        #[cfg_attr(feature = "en-US", doc = "database port 1 ~ 65535")]
        #[structopt(short = "P", long)]
        port: Option<u16>,

        #[cfg_attr(feature = "zh-CN", doc = "数据库名称")]
        #[cfg_attr(feature = "en-US", doc = "database name")]
        #[structopt(short, long)]
        db: Option<String>,

        #[cfg_attr(feature = "zh-CN", doc = "用户名")]
        #[cfg_attr(feature = "en-US", doc = "user name")]
        #[structopt(short, long)]
        user: Option<String>,

        #[cfg_attr(feature = "zh-CN", doc = "密码")]
        #[cfg_attr(feature = "en-US", doc = "password")]
        #[structopt(short = "pass", long)]
        password: Option<String>,

        #[cfg_attr(feature = "zh-CN", doc = "SSL 模式")]
        #[cfg_attr(feature = "en-US", doc = "SSL Mode")]
        #[structopt(long)]
        ssl_mode: Option<SslMode>,

        #[cfg_attr(feature = "zh-CN", doc = "SSL CA 文件路径")]
        #[cfg_attr(feature = "en-US", doc = "SSL CA file path")]
        #[structopt(long, parse(from_os_str))]
        ssl_ca: Option<std::path::PathBuf>,
    },
}

#[derive(Debug, StructOpt)]
pub enum StyleCmd {
    #[cfg_attr(feature = "zh-CN", doc = "配置打印表格样式")]
    #[cfg_attr(feature = "en-US", doc = "config table style")]
    Table {
        #[cfg_attr(
            feature = "zh-CN",
            doc = "选项: AsciiFull AsciiMd Utf8Full Utf8HBorderOnly"
        )]
        #[cfg_attr(
            feature = "en-US",
            doc = "choices: AsciiFull AsciiMd Utf8Full Utf8HBorderOnly"
        )]
        style: TableStyle,
    },
}

impl DCliCommand {
    pub async fn run(&self, config: &mut Config) -> Result<()> {
        match self {
            DCliCommand::Style { cmd } => {
                match cmd {
                    StyleCmd::Table { style } => {
                        config.table_style = style.clone();
                        config.save()?;
                    }
                };
                Ok(())
            }
            DCliCommand::Conn { profile } => {
                let profile = config.try_get_profile(profile)?;
                let mut sys_cmd = profile.cmd(false);
                let child = sys_cmd
                    .spawn()
                    .with_context(|| fl!("launch-process-failed"))?;
                child.wait_with_output().unwrap();
                Ok(())
            }
            DCliCommand::Exec { profile, command } => {
                let profile = config.try_get_profile(profile)?;
                let pool = connect(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                for sql in to_execute.split(";") {
                    if !sql.is_empty() {
                        let output: QueryOutput = sqlx::query(sql).fetch_all(&pool).await?.into();
                        println!("{}", output.to_print_table(&config));
                    }
                }
                pool.close().await;
                Ok(())
            }
            DCliCommand::Profile { cmd } | DCliCommand::P { cmd } => {
                match cmd {
                    ProfileCmd::List => {
                        let mut table = config.new_table();
                        table.set_header(vec!["name", "user", "host", "port", "database", "uri"]);
                        for (p_name, profile) in &config.profiles {
                            table.add_row(vec![
                                p_name,
                                &profile.user.clone().unwrap_or_default(),
                                &profile.host,
                                &profile.port.to_string(),
                                &profile.db.clone(),
                                &profile.uri(),
                            ]);
                        }
                        println!("{}", table);
                    }
                    ProfileCmd::Add {
                        name,
                        force,
                        profile,
                    } => {
                        if let Ok(_) = config.try_get_profile(name) {
                            if !force {
                                Err(anyhow!(fl!("profile-existed", name = name.clone())))?;
                            }
                        } else {
                            let mut cp = profile.clone();
                            cp.name = name.clone();
                            config.profiles.insert(name.clone(), cp);
                            config.save()?;
                            println!("{}", fl!("profile-saved"));
                        }
                    }
                    ProfileCmd::Del { profile } => {
                        let deleted = config.profiles.remove(profile);
                        if deleted.is_none() {
                            Err(anyhow!(fl!("profile-saved")))?;
                        } else {
                            config.save()?;
                            println!("{}", fl!("profile-deleted"));
                        }
                    }
                    ProfileCmd::Set {
                        name,
                        host,
                        port,
                        db,
                        user,
                        password,
                        ssl_mode,
                        ssl_ca,
                    } => {
                        let mut profile = config.try_get_profile(name)?.clone();
                        if let Some(host) = host {
                            profile.host = host.to_string();
                        }
                        if let Some(port) = port {
                            profile.port = port.clone();
                        }
                        if let Some(db) = db {
                            profile.db = db.clone()
                        }
                        if user.is_some() {
                            profile.user = user.clone()
                        }
                        if password.is_some() {
                            profile.password = password.clone()
                        }
                        if ssl_mode.is_some() {
                            profile.ssl_mode = ssl_mode.clone()
                        }
                        if ssl_ca.is_some() {
                            profile.ssl_ca = ssl_ca.clone()
                        }
                        config.try_set_profile(name, profile)?;
                        config.save()?;
                        println!("{}", fl!("profile-updated", name = name.clone()));
                    }
                }
                Ok(())
            }
            DCliCommand::Shell { profile } => shell::Shell::run(config, profile).await,
        }
    }
}

pub struct QueryOutput {
    rows: Vec<MySqlRow>,
}

impl From<Vec<MySqlRow>> for QueryOutput {
    fn from(rows: Vec<MySqlRow>) -> Self {
        Self { rows }
    }
}

impl QueryOutput {
    pub fn to_print_table(&self, config: &Config) -> Table {
        if self.rows.is_empty() {
            return config.new_table();
        }
        let header = self.rows.first().unwrap();
        let header_cols = header.columns();
        if header_cols.is_empty() {
            return config.new_table();
        }
        let mut table = config.new_table();
        table.set_header(header_cols.iter().map(|col| col.name()));
        self.rows.iter().for_each(|row| {
            table.add_row(row.columns().iter().map(|col| {
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
