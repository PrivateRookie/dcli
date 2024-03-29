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
    path::PathBuf,
    process::Stdio,
    str::FromStr,
};
use structopt::StructOpt;

use crate::fl;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub table_style: TableStyle,
    pub lang: Option<Lang>,
    pub debug: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Lang {
    #[serde(rename = "en-US")]
    EnUS,
    #[serde(rename = "zh-CN")]
    ZhCN,
}

impl Default for Lang {
    fn default() -> Self {
        Lang::EnUS
    }
}

impl FromStr for Lang {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_ascii_lowercase();
        if lower.starts_with("en-us") {
            Ok(Lang::EnUS)
        } else if lower.starts_with("zh-cn") {
            Ok(Lang::ZhCN)
        } else {
            Err(anyhow!(fl!("invalid-value", val = s)))
        }
    }
}

impl ToString for Lang {
    fn to_string(&self) -> String {
        match self {
            Lang::EnUS => "en-US",
            Lang::ZhCN => "zh-CN",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
pub struct Profile {
    #[structopt(skip)]
    pub name: String,

    #[cfg_attr(feature = "zh-CN", doc = "数据库 hostname, IPv6地址请使用'[]'包围")]
    #[cfg_attr(
        feature = "en-US",
        doc = "database hostname, IPv6 should be surrounded by '[]'"
    )]
    #[structopt(short = "h", long, default_value = "localhost")]
    pub host: String,

    #[cfg_attr(feature = "zh-CN", doc = "数据库 port 1 ~ 65535")]
    #[cfg_attr(feature = "en-US", doc = "database port 1 ~ 65535")]
    #[structopt(default_value = "3306", short = "P", long)]
    pub port: u16,

    #[cfg_attr(feature = "zh-CN", doc = "数据库名称")]
    #[cfg_attr(feature = "en-US", doc = "database name")]
    pub db: String,

    #[cfg_attr(feature = "zh-CN", doc = "用户名")]
    #[cfg_attr(feature = "en-US", doc = "user name")]
    #[structopt(short, long)]
    pub user: Option<String>,

    #[cfg_attr(feature = "zh-CN", doc = "密码")]
    #[cfg_attr(feature = "en-US", doc = "password")]
    #[structopt(short = "pass", long)]
    pub password: Option<String>,

    #[cfg_attr(feature = "zh-CN", doc = "SSL 模式")]
    #[cfg_attr(feature = "en-US", doc = "SSL Mode")]
    #[structopt(long)]
    pub ssl_mode: Option<SslMode>,

    #[cfg_attr(feature = "zh-CN", doc = "SSL CA 文件路径")]
    #[cfg_attr(feature = "en-US", doc = "SSL CA file path")]
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
            _ => return Err(anyhow!(fl!("invalid-value", val = s))),
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
            _ => return Err(anyhow!(fl!("invalid-value", val = s))),
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
        uri.push_str(&format!("/{}", self.db));
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
        command.args(&["--database", &self.db]);
        command
    }

    pub fn load_or_create_history(&self) -> Result<PathBuf> {
        let mut path = PathBuf::from(std::env::var("HOME").with_context(|| fl!("home-not-set"))?);
        path.push(".dcli");
        path.push("history");
        if !path.exists() {
            std::fs::create_dir_all(&path).with_context(|| fl!("create-his-dir-failed"))?
        }
        path.push(format!("{}_history.txt", self.name));
        if !path.exists() {
            std::fs::File::create(&path)
                .with_context(|| fl!("create-his-file-failed", name = self.name.clone()))?;
        }
        Ok(path)
    }
}

impl Config {
    pub fn config_path() -> Result<String> {
        let home = std::env::var("HOME").with_context(|| fl!("home-not-set"))?;
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
                .with_context(|| fl!("open-config-failed", file = file.to_str().unwrap_or("")))?
                .read_to_string(&mut content)
                .unwrap();
            let config: Config =
                toml::from_str(&content).with_context(|| fl!("ser-config-failed"))?;
            Ok(config)
        } else {
            println!(
                "{}",
                fl!("create-config-file", file = file.to_str().unwrap())
            );
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let mut file =
            File::create(&path).with_context(|| fl!("open-config-failed", file = path))?;
        let tmp_value = toml::Value::try_from(self).unwrap();
        let config_str = toml::to_string_pretty(&tmp_value).unwrap();
        file.write_all(config_str.as_bytes())
            .with_context(|| fl!("save-config-filed"))?;
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
            let table_str = table.to_string();
            Err(anyhow!(fl!(
                "profile-not-found",
                name = name,
                table = table_str
            )))
        }
    }

    pub fn try_set_profile(&mut self, name: &str, new_profile: Profile) -> Result<()> {
        self.try_get_profile(name)?;
        self.profiles.insert(name.to_string(), new_profile);
        Ok(())
    }
}
