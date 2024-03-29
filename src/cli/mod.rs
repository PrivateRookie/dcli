use crate::{
    config::{Config, Lang, Profile, SslMode, TableStyle},
    mysql::Session,
    output::Format,
    utils::read_file,
};
use crate::{fl, query::QueryPlan};
use anyhow::{anyhow, Context, Result};
use http::serve_plan;
use std::{collections::HashMap, io::Write};
use structopt::StructOpt;

mod http;
pub mod shell;

#[cfg_attr(feature = "zh-CN", doc = "数据库连接工具")]
#[cfg_attr(feature = "en-US", doc = "database connection manage tool")]
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

    #[cfg_attr(feature = "zh-CN", doc = "导出查询结果")]
    #[cfg_attr(feature = "en-US", doc = "export query output")]
    Export {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        #[structopt(short, long)]
        profile: String,

        #[cfg_attr(feature = "zh-CN", doc = "输出格式: csv, json, yaml, toml, pickle")]
        #[cfg_attr(
            feature = "en-US",
            doc = "output format: csv, json, yaml, toml, pickle"
        )]
        #[structopt(short, long, default_value = "csv")]
        format: Format,

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

    #[cfg_attr(feature = "zh-CN", doc = "运行一个 HTTP　服务器以展示，下载数据")]
    #[cfg_attr(
        feature = "en-US",
        doc = "run a HTTP server to display or download data"
    )]
    Serve {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        #[structopt(short, long)]
        profile: String,

        #[cfg_attr(feature = "zh-CN", doc = "port 1 ~ 65535")]
        #[cfg_attr(feature = "en-US", doc = "port 1 ~ 65535")]
        #[structopt(default_value = "3030", short = "P", long)]
        port: u16,

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

    Plan {
        plan: String,
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
    #[cfg_attr(feature = "zh-CN", doc = "设置语言")]
    #[cfg_attr(feature = "en-US", doc = "set language")]
    Lang {
        #[cfg_attr(feature = "zh-CN", doc = "语言, 可选 en-US, zh-CN")]
        #[cfg_attr(feature = "en-US", doc = "lang, options: en-US, zh-CN")]
        name: Option<Lang>,
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
                    StyleCmd::Lang { name } => {
                        config.lang = name.clone();
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
                let session = Session::connect_with(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                for sql in to_execute.split(';') {
                    if !sql.is_empty() {
                        let output = session.query(sql).await?;
                        println!("{}", output.to_print_table(&config));
                    }
                }
                session.close().await;
                Ok(())
            }
            DCliCommand::Export {
                profile,
                command,
                format,
            } => {
                let profile = config.try_get_profile(profile)?;
                let session = Session::connect_with(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                let to_execute = to_execute
                    .split(';')
                    .filter(|sql| !sql.is_empty())
                    .collect::<Vec<&str>>();
                if to_execute.is_empty() {
                    return Err(anyhow!(fl!("empty-input")));
                } else if to_execute.len() > 1 {
                    return Err(anyhow!(fl!("too-many-input")));
                } else {
                    let output = session.query(to_execute.first().unwrap()).await?;
                    match format {
                        Format::Csv => {
                            let out = output.to_csv()?;
                            println!("{}", out);
                        }
                        Format::Json => {
                            let out = output.to_json()?;
                            println!("{}", out);
                        }
                        Format::Yaml => {
                            let out = output.to_yaml()?;
                            println!("{}", out);
                        }
                        Format::Toml => {
                            let out = output.to_toml()?;
                            println!("{}", out);
                        }
                        Format::Pickle => {
                            let mut stdout = std::io::stdout();
                            stdout.write_all(&output.to_pickle()?)?;
                            stdout.flush()?;
                        }
                    }
                    Ok(())
                }
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
                        if config.try_get_profile(name).is_ok() {
                            if !force {
                                return Err(anyhow!(fl!("profile-existed", name = name.clone())));
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
                            return Err(anyhow!(fl!("profile-saved")));
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
                            profile.port = *port;
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
            DCliCommand::Serve {
                profile,
                port,
                command,
            } => {
                let profile = config.try_get_profile(profile)?;
                let session = Session::connect_with(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                let to_execute = to_execute
                    .split(';')
                    .filter(|sql| !sql.is_empty())
                    .collect::<Vec<&str>>();
                if to_execute.is_empty() {
                    return Err(anyhow!(fl!("empty-input")));
                } else if to_execute.len() > 1 {
                    return Err(anyhow!(fl!("too-many-input")));
                } else {
                    let output = session.query(to_execute.first().unwrap()).await?;
                    http::serve(*port, output).await;
                    Ok(())
                }
            }
            DCliCommand::Plan { plan } => {
                let content = read_file(plan)?;
                let plan: QueryPlan = toml::from_str(&content)?;
                let mut plan_sessions: HashMap<String, Session> = HashMap::new();
                for p in plan.profiles() {
                    if let Ok(profile) = config.try_get_profile(&p) {
                        let session = Session::connect_with(profile).await?;
                        plan_sessions.insert(p, session);
                    }
                }
                serve_plan(plan, plan_sessions).await;
                Ok(())
            }
        }
    }
}
