use crate::commands::status::parse_gitignore;
use crate::objects::blob;
use crate::others::file_altering;
use crate::others::index::{Index, ObjectInfo};
use anyhow::Result;
use std::fs::read_to_string;
use std::path::PathBuf;
pub fn read_file_content(file_name: &str) -> Result<Vec<u8>> {
    let content = read_to_string(file_name)?;
    Ok(content.into_bytes())
}

pub fn verifcation(obj: &ObjectInfo) -> Result<usize> {
    let index_content = String::from_utf8(read_file_content(".vcs/index")?)?;
    let mut line_number = 0;
    let mut flag = false;
    for line in index_content.lines() {
        line_number += 1;
        let parsed_obj = ObjectInfo::from_pretty_print(line)?;

        if parsed_obj.path == obj.path {
            if !obj.path.exists() {
                // we can add freely the obj
                return Ok(1); // 1 if it does not exist
            }
            if parsed_obj.ctime != obj.ctime || parsed_obj.mtime != obj.mtime {
                // i need to delete the line from the file index
                return Ok(line_number + 150);
            }
            flag = true;
        }
    }
    if obj.path.exists() && !flag {
        return Ok(1);
    }
    if line_number == 0 {
        return Ok(1);
    }
    Ok(0)
}

pub fn add_file(file_name: &str) -> Result<()> {
    let content = read_file_content(file_name)?;
    let blobs = blob::Blob::new(content);
    blobs.create_blob()?;
    let path = PathBuf::from(file_name);
    let mut index = Index::new();
    let obj = ObjectInfo::new("blob", &path, &blobs.get_hash())?;

    match verifcation(&obj)? {
        0 => {
            println!("The file was not modified");
        }
        1 => {
            //println!("First time you add this file");
            index.add_object(obj);
            index.save_index_file_append()?;
        }
        line_number => {
            file_altering::delete_nth_line(line_number - 151, ".vcs/index")?;
            // Delete the outdated entry and update the index
            index.add_object(obj);
            index.save_index_file_append()?;
        }
    }
    Ok(())
}

pub fn add_protocol(file_name: &str) -> Result<()> {
    let path = PathBuf::from(file_name);
    let ignore_patt = parse_gitignore()?;
    let mut filenames = String::new();
    if path.is_dir() {
        let temp = file_altering::get_all_filenames(file_name, &ignore_patt)?;
        filenames.push_str(&temp);
        for line in filenames.lines() {
            add_file(line)?;
        }
    } else if path.is_file() {
        add_file(file_name)?;
    } else {
        return Err(anyhow::anyhow!("Invalid file path: {}", file_name));
    }

    Ok(())
}
