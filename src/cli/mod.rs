use crate::{
    config::{Config, Profile, SslMode, TableStyle},
    mysql::connect,
    utils::read_file,
};
use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use comfy_table::*;
// use rust_decimal::Decimal;
use sqlx::{
    mysql::MySqlRow, types::time::Date, types::time::Time, Column, Row, TypeInfo, Value, ValueRef,
};
use structopt::StructOpt;

pub mod shell;

#[derive(Debug, StructOpt)]
#[structopt(name = "dcli", about = "数据连接工具.")]
pub enum DCliCommand {
    /// 配置相关命令 profile 命令别名
    P {
        #[structopt(subcommand)]
        cmd: ProfileCmd,
    },
    /// 配置相关命令
    Profile {
        #[structopt(subcommand)]
        cmd: ProfileCmd,
    },
    /// 显示样式相关命令
    Style {
        #[structopt(subcommand)]
        cmd: StyleCmd,
    },
    /// 使用 `mysql` 命令连接到 mysql
    Conn {
        /// 连接配置名称
        profile: String,
    },
    /// 使用一个配置运行命令
    Exec {
        /// 配置名
        #[structopt(short, long)]
        profile: String,

        /// 命令 使用 @<文件路径> 读取 SQL 文件内容作为输入
        command: Vec<String>,
    },

    /// 运行连接到 mysql 的 shell
    Shell {
        /// 配置名
        #[structopt(short, long)]
        profile: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum ProfileCmd {
    /// 列出所有配置
    List,
    /// 添加一个配置
    Add {
        /// 配置名称
        name: String,

        /// 是否强制覆盖
        #[structopt(short, long)]
        force: bool,

        #[structopt(flatten)]
        profile: Profile,
    },
    /// 删除一个配置
    Del {
        /// 配置名
        profile: String,
    },
    /// 修改已有配置
    Set {
        /// 配置名称
        name: String,

        /// 数据库 hostname, IPv6地址请使用'[]'包围
        #[structopt(short = "h", long)]
        host: Option<String>,

        /// 数据库 port 0 ~ 65536
        #[structopt(short = "P", long)]
        port: Option<u16>,

        /// 数据库名称
        #[structopt(short, long)]
        db: Option<String>,

        /// 用户名
        #[structopt(short, long)]
        user: Option<String>,

        /// 密码
        #[structopt(short = "pass", long)]
        password: Option<String>,

        /// SSL 模式
        #[structopt(long)]
        ssl_mode: Option<SslMode>,

        /// SSL CA 文件路径
        #[structopt(long, parse(from_os_str))]
        ssl_ca: Option<std::path::PathBuf>,
    },
}

#[derive(Debug, StructOpt)]
pub enum StyleCmd {
    /// 配置打印表格样式
    Table {
        /// 选项 AsciiFull AsciiMd Utf8Full Utf8HBorderOnly
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
                let child = sys_cmd.spawn().with_context(|| "无法启动程序!")?;
                child.wait_with_output().unwrap();
                Ok(())
            }
            DCliCommand::Exec { profile, command } => {
                let profile = config.try_get_profile(profile)?;
                let mut conn = connect(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                for sql in to_execute.split(";") {
                    if !sql.is_empty() {
                        let output: QueryOutput =
                            sqlx::query(sql).fetch_all(&mut conn).await?.into();
                        println!("{}", output.to_print_table(&config));
                    }
                }
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
                                Err(anyhow!(format!("{} 配置已存在!", name)))?;
                            }
                        } else {
                            let mut cp = profile.clone();
                            cp.name = name.clone();
                            config.profiles.insert(name.clone(), cp);
                            config.save()?;
                            println!("配置已保存.");
                        }
                    }
                    ProfileCmd::Del { profile } => {
                        let deleted = config.profiles.remove(profile);
                        if deleted.is_none() {
                            Err(anyhow!("未找到配置."))?;
                        } else {
                            config.save()?;
                            println!("配置已删除.");
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
                        println!("{} 配置已更新", name);
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
