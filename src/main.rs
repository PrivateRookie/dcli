use comfy_table::Table;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Read};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "dcli", about = "数据连接工具.")]
enum Command {
    /// 打开连接
    Connect {
        /// 连接配置名称
        profile: String,
    },

    /// 列出所有连接
    List,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Profile {
    db: String,
    user: String,
    password: Option<String>,
    host: String,
    port: u16,
}

impl Profile {
    pub fn uri(&self) -> String {
        let pass = match &self.password {
            Some(pass) => format!(":{}", pass),
            None => String::new(),
        };
        format!(
            "mysql://{}{}@{}:{}/{}",
            self.user, pass, self.host, self.port, self.db
        )
    }

    pub fn cmd(&self) -> std::process::Command {
        let mut command = std::process::Command::new("mysql");
        command.args(&[
            "-u",
            &self.user,
            "--host",
            &self.host,
            "--port",
            &self.port.to_string(),
        ]);

        if let Some(ref pass) = self.password {
            command.arg(&format!("-p{}", pass));
        }
        command.arg(&self.db);
        command
    }
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| format!("$HOME is not set"))?;
        let mut file = std::path::Path::new(&home).to_path_buf();
        file.push(".config");
        file.push("dcli.toml");
        let mut content = String::new();
        std::fs::File::open(&file)
            .expect(&format!(
                "fail to load config file {}",
                file.to_str().unwrap()
            ))
            .read_to_string(&mut content)
            .expect("read failed");
        let config: Config =
            toml::from_str(&content).map_err(|e| format!("fail to read config {}", e))?;
        Ok(config)
    }
}

fn main() {
    let cmd = Command::from_args();
    let config = Config::load().unwrap();
    match cmd {
        Command::List => {
            let mut table = Table::new();
            table.set_header(vec!["name", "user", "host", "port", "database", "uri"]);
            for (p_name, profile) in &config.profiles {
                table.add_row(vec![
                    p_name,
                    &profile.user,
                    &profile.host,
                    &profile.port.to_string(),
                    &profile.db,
                    &profile.uri(),
                ]);
            }
            println!("{}", table);
        }
        Command::Connect { profile } => {
            let msg = format!(
                "can't find {}, available option are: {:?}",
                &profile,
                config.profiles.keys()
            );
            let profile = config.profiles.get(&profile).expect(&msg);
            let mut cmd = profile.cmd();
            let child = cmd.spawn().expect("run failed");
            child.wait_with_output().unwrap();
        }
    }
}
