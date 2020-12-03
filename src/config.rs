use anyhow::{anyhow, Context, Result};
use comfy_table::{
    presets::{ASCII_FULL, ASCII_MARKDOWN, UTF8_FULL, UTF8_HORIZONTAL_BORDERS_ONLY},
    ContentArrangement, Table,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    process::Stdio,
    str::FromStr,
};
use structopt::StructOpt;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub table_style: TableStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub struct Profile {
    /// 数据库 hostname, IPv6地址请使用带'[]'包围
    #[structopt(short = "h", long, default_value = "localhost")]
    pub host: String,

    /// 数据库 port 0 ~ 65536
    #[structopt(default_value = "3306", short = "P", long)]
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

    /// SSL 模式
    #[structopt(long)]
    pub ssl_mode: Option<SslMode>,

    /// SSL CA 文件路径
    #[structopt(long, parse(from_os_str))]
    pub ssl_ca: Option<std::path::PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslMode {
    Disabled,
    Preferred,
    Required,
    VerifyCa,
    VerifyIdentity,
}

impl Default for SslMode {
    fn default() -> Self {
        SslMode::Preferred
    }
}

impl FromStr for SslMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match &*s.to_ascii_lowercase() {
            "disabled" => SslMode::Disabled,
            "preferred" => SslMode::Preferred,
            "required" => SslMode::Required,
            "verify_ca" => SslMode::VerifyCa,
            "verify_identity" => SslMode::VerifyIdentity,
            _ => Err(anyhow!(format!("无效值: {:?}", s)))?,
        };
        Ok(val)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableStyle {
    AsciiFull,
    AsciiMd,
    Utf8Full,
    Utf8HBorderOnly,
}

impl Default for TableStyle {
    fn default() -> Self {
        TableStyle::Utf8Full
    }
}

impl FromStr for TableStyle {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = match &*s.to_ascii_lowercase() {
            "asciifull" => TableStyle::AsciiFull,
            "asciimd" => TableStyle::AsciiMd,
            "utf8full" => TableStyle::Utf8Full,
            "utf8hborderonly" => TableStyle::Utf8HBorderOnly,
            _ => Err(anyhow!(format!("无效值: {:?}", s)))?,
        };
        Ok(val)
    }
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
        if self.user.is_none() && self.password.is_none() {
            uri.push_str(&format!("{}:{}", self.host, self.port));
        } else {
            uri.push_str(&format!("@{}:{}", self.host, self.port));
        }
        if let Some(db) = &self.db {
            uri.push_str(&format!("/{}", db));
        }
        uri
    }

    pub fn cmd(&self, piped: bool) -> std::process::Command {
        let mut command = std::process::Command::new("mysql");
        if piped {
            command
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        }
        if let Some(user) = &self.user {
            command.args(&["--user", user]);
        }
        if let Some(pass) = &self.password {
            command.arg(&format!("--password={}", pass));
        }
        command.args(&["--host", &self.host, "--port", &self.port.to_string()]);
        if let Some(db) = &self.db {
            command.args(&["--database", db]);
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
                .unwrap();
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
        // NOTE toml 系列化问题
        let tmp_value = toml::Value::try_from(self).unwrap();
        let config_str = toml::to_string_pretty(&tmp_value).unwrap();
        file.write_all(config_str.as_bytes())
            .with_context(|| "无法写入配置文件")?;
        Ok(())
    }

    pub fn new_table(&self) -> Table {
        let mut table = Table::new();
        let preset = match self.table_style {
            TableStyle::AsciiFull => ASCII_FULL,
            TableStyle::AsciiMd => ASCII_MARKDOWN,
            TableStyle::Utf8Full => UTF8_FULL,
            TableStyle::Utf8HBorderOnly => UTF8_HORIZONTAL_BORDERS_ONLY,
        };
        table
            .load_preset(preset)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table
    }

    pub fn try_get_profile(&self, name: &str) -> Result<&Profile> {
        if let Some(profile) = self.profiles.get(name) {
            Ok(profile)
        } else {
            let mut table = self.new_table();
            table.set_header(vec!["name"]);
            self.profiles.keys().into_iter().for_each(|key| {
                table.add_row(vec![key]);
            });
            Err(anyhow!(format!(
                "未找到配置文件 {}, 请在以下选项中选择\n{}",
                name, table
            )))
        }
    }
}
