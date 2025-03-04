use crate::others::compression;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
pub fn read_file_content(file_path: &str) -> Result<Vec<u8>> {
    let content = fs::read(file_path).context(format!("Failed to read file: {}", file_path))?;
    Ok(content)
}

pub fn create_object_path(obj_hash: &str) -> PathBuf {
    let mut path = PathBuf::from(".vcs/objects/");

    let subfolder = &obj_hash[0..2];

    let filename = &obj_hash[2..];

    path.push(subfolder);
    path.push(filename);

    path
}

pub fn dec_obj(obj_hash: &str) -> Result<String> {
    let file_content = read_file_content(create_object_path(obj_hash).to_str().unwrap())?;
    let decompressed_msg = compression::decompress(&file_content)?;
    Ok(String::from_utf8(decompressed_msg)?)
}

pub fn get_info(obj_hash: &str) -> Result<()> {
    let file_content = read_file_content(create_object_path(obj_hash).to_str().unwrap())?;

    let decompressed_msg = compression::decompress(&file_content)?;

    println!("{}", String::from_utf8(decompressed_msg)?);

    Ok(())
}
