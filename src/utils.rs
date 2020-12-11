use crate::fl;
use anyhow::{Context, Result};
use std::io::Read;

pub fn read_file(path: &str) -> Result<String> {
    let mut file =
        std::fs::File::open(path).with_context(|| fl!("open-file-failed", file = path))?;
    let mut content = String::new();
    file.read_to_string(&mut content)
        .with_context(|| fl!("read-file-failed", file = path))?;
    Ok(content)
}
