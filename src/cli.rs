use crate::config::{Config, Profile};
use anyhow::{anyhow, Context, Result};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dcli", about = "数据连接工具.")]
pub enum Command {
    /// 使用一个配置打开连接
    Conn {
        /// 连接配置名称
        profile: String,
    },

    /// 使用一个配置执行命令
    Exec {
        /// 配置名
        #[structopt(short, long)]
        profile: String,

        /// 命令
        command: Vec<String>,
    },

    /// 列出所有配置
    List,

    /// 添加一个配置
    Add(AddProfile),

    /// 删除一个配置
    Delete {
        /// 配置名
        name: String,
    },
}

impl Command {
    pub fn run(self, config: &mut Config) -> Result<()> {
        match self {
            Command::List => {
                let mut table = default_table();
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
            Command::Conn { ref profile } => {
                if let Some(profile) = config.profiles.get(profile) {
                    let mut cmd = profile.cmd(false);
                    let child = cmd.spawn().expect("run failed");
                    child.wait_with_output().unwrap();
                } else {
                    let mut table = default_table();
                    table.set_header(vec!["name"]);
                    config.profiles.keys().into_iter().for_each(|key| {
                        table.add_row(vec![key]);
                    });
                    println!("未找到配置文件 {}, 请在以下选项中选择\n {}", profile, table);
                }
            }
            Command::Add(AddProfile {
                name,
                host,
                port,
                user,
                password,
                db,
                force,
            }) => {
                let profile = Profile {
                    host,
                    port,
                    user,
                    password,
                    db,
                };
                let exits = config.profiles.contains_key(&name);
                if !force && exits {
                    Err(anyhow!(format!("{} 配置已存在!", name)))?;
                } else {
                    config.profiles.insert(name, profile);
                    config.save()?;
                    println!("配置已保存.");
                }
            }
            Command::Delete { name } => {
                let deleted = config.profiles.remove(&name);
                if deleted.is_none() {
                    Err(anyhow!("未找到配置."))?;
                } else {
                    config.save()?;
                    println!("配置已删除.");
                }
            }
            Command::Exec { profile, command } => {
                if let Some(profile) = config.profiles.get(&profile) {
                    let mut cmd = profile.cmd(true);
                    cmd.arg(&format!("--execute={}", command.join(" ")));
                    let child = cmd.spawn().with_context(|| "无法运行")?;
                    let output = child.wait_with_output().unwrap();
                    println!("{}", String::from_utf8(output.stdout).unwrap());
                } else {
                    let mut table = default_table();
                    table.set_header(vec!["name"]);
                    config.profiles.keys().into_iter().for_each(|key| {
                        table.add_row(vec![key]);
                    });
                    println!("未找到配置文件 {}, 请在以下选项中选择\n {}", profile, table);
                }
            }
        }
        Ok(())
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

fn default_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}
