use crate::others::compression;
use crate::others::hash_function::calculate_hash;
use anyhow::Result;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct Blob {
    pub id: String, // SHA1 HASH
    pub content: Vec<u8>,
}

impl Blob {
    pub fn new(file_content: Vec<u8>) -> Self {
        let id = calculate_hash(&file_content);
        Blob {
            id,
            content: file_content,
        }
    }
    pub fn get_hash(self) -> String {
        self.id
    } // the object is destroyed after the use of this fucntion
    pub fn create_blob(&self) -> Result<()> {
        let vcs_objects_path: PathBuf = Path::new(".vcs").join("objects");
        if !vcs_objects_path.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        let subfolder = vcs_objects_path.join(&self.id[0..2]);

        if !subfolder.exists() {
            fs::create_dir_all(&subfolder)?;
        }

        let file_path = subfolder.join(&self.id[2..]);

        let compressed_content = compression::compress(&self.content)?;

        let mut file = fs::File::create(file_path)?;
        file.write_all(&compressed_content)?;

        Ok(())
    }
}
