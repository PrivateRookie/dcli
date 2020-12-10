use anyhow::{Context, Result};
use std::io::Read;

pub fn read_file(path: &str) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("无法打开文件 {}", path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| "无法读取文件")?;
    Ok(content)
}
