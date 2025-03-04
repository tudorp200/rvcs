use anyhow::{Context, Result};
use std::fs;

pub fn read_file_content(file_path: &str) -> Result<Vec<u8>> {
    let content = fs::read(file_path).context(format!("Failed to read file: {}", file_path))?;
    Ok(content)
}

pub fn get_info() -> Result<()> {
    let file_content = read_file_content(".vcs/index")?;

    println!("{}", String::from_utf8(file_content)?);

    Ok(())
}
