use crate::{
    config::{Config, Lang, Profile, SslMode, TableStyle},
    mysql::connect,
    output::Format,
    utils::read_file,
};
use crate::{fl, output::QueryOutput};
use anyhow::{anyhow, Context, Result};
use structopt::StructOpt;

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

    Export {
        #[cfg_attr(feature = "zh-CN", doc = "连接配置名称")]
        #[cfg_attr(feature = "en-US", doc = "profile name")]
        #[structopt(short, long)]
        profile: String,

        #[cfg_attr(feature = "zh-CN", doc = "输出格式: json, yaml, toml, pickle")]
        #[cfg_attr(feature = "en-US", doc = "output format: json, yaml, toml, pickle")]
        #[structopt(short, long, default_value = "json")]
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
            DCliCommand::Export {
                profile,
                command,
                format,
            } => {
                let profile = config.try_get_profile(profile)?;
                let pool = connect(&profile).await?;
                let to_execute = if command.len() == 1 && command.first().unwrap().starts_with('@')
                {
                    read_file(&command.first().unwrap()[1..])?
                } else {
                    command.join(" ")
                };
                let to_execute = to_execute
                    .split(";")
                    .filter(|sql| !sql.is_empty())
                    .collect::<Vec<&str>>();
                if to_execute.len() == 0 {
                    Err(anyhow!(fl!("empty-input")))?
                } else if to_execute.len() > 1 {
                    Err(anyhow!(fl!("too-many-input")))?
                } else {
                    let output: QueryOutput = sqlx::query(to_execute.first().unwrap())
                        .fetch_all(&pool)
                        .await?
                        .into();
                    match format {
                        Format::Json => {
                            let out_str = serde_json::to_string(&output)
                                .with_context(|| fl!("serialize-output-failed"))?;
                            println!("{}", out_str);
                        }
                        Format::Yaml => {
                            let out_str = serde_yaml::to_string(&output)
                                .with_context(|| fl!("serialize-output-failed"))?;
                            println!("{}", out_str);
                        }
                        Format::Toml => {
                            let out_str = toml::to_string_pretty(&output)
                                .with_context(|| fl!("serialize-output-failed"))?;
                            println!("{}", out_str);
                        }
                        Format::Pickle => {
                            let mut stdout = std::io::stdout();
                            serde_pickle::to_writer(&mut stdout, &output, false)
                                .with_context(|| fl!("serialize-output-failed"))?;
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
