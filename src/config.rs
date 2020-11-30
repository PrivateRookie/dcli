use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub host: String,
    pub port: u16,
    pub db: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Profile {
    pub fn uri(&self) -> String {
        let mut uri = String::from("mysql://");
        if let Some(user) = &self.user {
            uri.push_str(user)
        }
        if let Some(pass) = &self.password {
            uri.push_str(&format!(":{}", pass))
        }
        uri.push_str(&format!("{}:{}", self.host, self.port));
        if let Some(db) = &self.db {
            uri.push_str(&format!("/{}", db));
        }
        uri
    }

    pub fn cmd(&self) -> std::process::Command {
        let mut command = std::process::Command::new("mysql");
        if let Some(user) = &self.user {
            command.args(&["--user", user]);
        }
        if let Some(pass) = &self.password {
            command.arg(&format!("--password={}", pass));
        }
        command.args(&["--host", &self.host, "--port", &self.port.to_string()]);
        if let Some(db) = &self.db {
            command.arg(db);
        }
        command
    }
}

impl Config {
    pub fn config_path() -> Result<String> {
        let home = std::env::var("HOME").with_context(|| "未设置 $HOME 环境变量")?;
        let mut file = std::path::Path::new(&home).to_path_buf();
        file.push(".config");
        file.push("dcli.toml");
        Ok(file.to_str().unwrap().to_string())
    }

    pub fn load() -> Result<Self> {
        let path_str = Self::config_path()?;
        let file = std::path::Path::new(&path_str);
        if file.exists() {
            let mut content = String::new();
            File::open(&file)
                .with_context(|| format!("fail to load config file {}", file.to_str().unwrap()))?
                .read_to_string(&mut content)
                .expect("read failed");
            let config: Config = toml::from_str(&content).with_context(|| "无法打开配置文件")?;
            Ok(config)
        } else {
            println!("未找到配置文件, 创建默认配置 {}", file.to_str().unwrap());
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let mut file = File::create(path).with_context(|| "无法打开配置文件")?;
        let config_str = toml::to_string_pretty(self).unwrap();
        file.write_all(config_str.as_bytes())
            .with_context(|| "无法写入配置文件")?;
        Ok(())
    }
}
