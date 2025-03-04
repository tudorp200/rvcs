use crate::commands::dec_object::dec_obj;
use crate::others::compression;
use crate::others::hash_function::calculate_hash;
use crate::others::index;
use anyhow::Result;
use std::fs::read_to_string;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
pub struct Tree {
    pub id: String,
}

pub fn read_file_content(file_name: &str) -> Result<Vec<u8>> {
    let content = read_to_string(file_name)?;
    Ok(content.into_bytes())
}

impl Tree {
    pub fn new() -> Result<Self> {
        let index_content = read_file_content(".vcs/index")?;
        let header = format!("tree{}\0", index_content.len());
        let mut content = header.into_bytes();
        content.extend_from_slice(&index_content);
        let id = calculate_hash(&content);
        Ok(Tree { id })
    }
    pub fn create_tree(&self) -> Result<()> {
        let vcs_objects_path: PathBuf = Path::new(".vcs").join("objects");
        if !vcs_objects_path.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        let subfolder = vcs_objects_path.join(&self.id[0..2]);

        if !subfolder.exists() {
            fs::create_dir_all(&subfolder)?;
        }

        let file_path = subfolder.join(&self.id[2..]);
        let index_content = read_file_content(".vcs/index")?;
        let header = format!("tree{}\0", index_content.len());
        let mut content = header.into_bytes();
        content.extend_from_slice(&index_content);

        let compressed_content = compression::compress(&index_content)?;

        let mut file = fs::File::create(file_path)?;
        file.write_all(&compressed_content)?;

        Ok(())
    }

    pub fn merge_tree(tree1: &str, tree2: &str) -> Result<index::Index> {
        let tree1_content = dec_obj(tree1)?;
        let tree2_content = dec_obj(tree2)?;

        let mut merged_index = index::Index::new();

        // Parse tree1 into the merged index
        for line in tree1_content.lines() {
            let obj_info = index::ObjectInfo::from_pretty_print(line)?;
            merged_index.add_object(obj_info);
        }

        // Parse tree2 and overwrite or add to the merged index
        for line in tree2_content.lines() {
            let obj_info = index::ObjectInfo::from_pretty_print(line)?;

            // If the object already exists, replace it
            merged_index.obj.retain(|obj| obj.path != obj_info.path);
            merged_index.add_object(obj_info);
        }

        Ok(merged_index)
    }

    pub fn new_tree_from_index(index: index::Index) -> Result<Self> {
        let index_content = Self::get_index_content(index)?;
        let header = format!("tree{}\0", index_content.len());
        let mut content = header.into_bytes();
        content.extend_from_slice(&index_content.into_bytes());
        let id = calculate_hash(&content);
        Ok(Tree { id })
    }

    pub fn get_index_content(index: index::Index) -> Result<String> {
        let mut index_content = String::new();
        for obj in index.obj {
            let obj_content = index::ObjectInfo::pretty_print(&obj);
            index_content.push_str(&obj_content);
            index_content.push('\n');
        }
        Ok(index_content)
    }

    pub fn create_tree_from_index(&self, index: index::Index) -> Result<()> {
        let vcs_objects_path: PathBuf = Path::new(".vcs").join("objects");
        if !vcs_objects_path.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        let subfolder = vcs_objects_path.join(&self.id[0..2]);

        if !subfolder.exists() {
            fs::create_dir_all(&subfolder)?;
        }
        let index_content = Self::get_index_content(index)?;

        let compressed_content = compression::compress(&index_content.into_bytes())?;

        let file_path = subfolder.join(&self.id[2..]);
        let mut file = fs::File::create(file_path)?;
        file.write_all(&compressed_content)?;

        Ok(())
    }
}
