use crate::{
    config::{Config, Profile, TableStyle},
    mysql::connect,
};
use anyhow::{anyhow, Context, Result};
use comfy_table::*;
use sqlx::{mysql::MySqlRow, Column, Row, TypeInfo, Value, ValueRef};
use structopt::StructOpt;

pub mod shell;

#[derive(Debug, StructOpt)]
#[structopt(name = "dcli", about = "数据连接工具.")]
pub enum DCliCommand {
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

        /// 命令
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
                let output: QueryOutput = sqlx::query(&command.join(" "))
                    .fetch_all(&mut conn)
                    .await?
                    .into();
                println!("{}", output.to_print_table(&config));
                Ok(())
            }
            DCliCommand::Profile { cmd } => {
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
                                &profile.db.clone().unwrap_or_default(),
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
                }
                Ok(())
            }
            DCliCommand::Shell { profile } => shell::Shell::run(config, profile).await,
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct AddProfile {
    /// 配置名称
    pub name: String,

    /// 数据库 hostname, IPv6请使用带'[]'包围的域名
    #[structopt(short = "h", long, default_value = "localhost")]
    pub host: String,

    /// 数据库 port 0 ~ 65536
    #[structopt(default_value = "3306", long)]
    pub port: u16,

    /// 数据库名称
    #[structopt(short, long)]
    pub db: Option<String>,

    /// 用户名
    #[structopt(short, long)]
    pub user: Option<String>,

    /// 密码
    #[structopt(short = "pass", long)]
    pub password: Option<String>,

    /// 是否强制覆盖
    #[structopt(short, long)]
    pub force: bool,
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
                    String::new()
                } else {
                    let ty_info = col.type_info();
                    if ty_info.name() == "TEXT" || ty_info.name() == "VARCHAR" {
                        val.decode::<String>()
                    } else {
                        val.decode::<i64>().to_string()
                    }
                };
                val
            }));
        });
        table
    }
}
