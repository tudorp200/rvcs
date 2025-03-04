use anyhow::{Context, Result};

use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};

use std::os::unix::fs::PermissionsExt;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug)]
pub struct ObjectInfo {
    pub obj_type: String,
    pub hash: String,
    pub ctime: u64,    // last time when the metadata chaged
    pub mtime: u64,    // last time the content changed
    pub path: PathBuf, // path to the file
    pub size: u64,     //size of the file
    pub permissions: u32,
}
#[derive(Debug)]
pub struct Index {
    pub obj: Vec<ObjectInfo>,
}

impl ObjectInfo {
    pub fn new(object_type: &str, path: &Path, hash: &str) -> Result<Self> {
        let metadata = fs::metadata(path).context("Failed to retrieve file metadata")?;

        let ctime = metadata
            .created()
            .unwrap_or(SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mtime = metadata
            .modified()
            .unwrap_or(SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let size = metadata.len();
        let permissions = metadata.permissions().mode();

        Ok(Self {
            obj_type: object_type.to_string(),
            hash: hash.to_string(),
            ctime,
            mtime,
            path: path.to_path_buf(),
            size,
            permissions,
        })
    }
    pub fn pretty_print(&self) -> String {
        format!(
            "{} {} {} {} {} {} {}",
            self.permissions,
            self.hash,
            self.obj_type,
            self.path.display(),
            self.size,
            self.ctime,
            self.mtime
        )
    }
    pub fn from_pretty_print(line: &str) -> Result<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(anyhow::anyhow!("Malformed index line: {}", line));
        }
        Ok(Self {
            obj_type: parts[2].to_string(),
            hash: parts[1].to_string(),
            permissions: parts[0].parse().context("Failed to parse permissions")?,
            ctime: parts[5].trim().parse().context("Failed to parse ctime")?,
            mtime: parts[6].trim().parse().context("Failed to parse mtime")?,
            path: PathBuf::from(parts[3]),
            size: parts[4].parse().context("Failed to parse size")?,
        })
    }
}

impl Index {
    pub fn new() -> Self {
        Index { obj: Vec::new() }
    }
    pub fn add_object(&mut self, object: ObjectInfo) {
        self.obj.push(object);
    }

    pub fn save_index_file_append(&self) -> Result<()> {
        let vcs_index_file = Path::new(".vcs");

        if !vcs_index_file.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        let file_path = vcs_index_file.join("index");

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path)
            .context("Failed to open or create the index file")?;

        let index_content = self
            .obj
            .iter()
            .map(|obj| obj.pretty_print() + "\n")
            .collect::<Vec<String>>()
            .join("\n")
            .into_bytes();

        file.write_all(&index_content)
            .context("Failed to append compressed content to the index file")?;

        Ok(())
    }

    pub fn save_index_file_truncate(&self) -> Result<()> {
        let vcs_index_file = Path::new(".vcs");

        if !vcs_index_file.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        let file_path = vcs_index_file.join("index");

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&file_path)
            .context("Failed to open or create the index file")?;

        let index_content = self
            .obj
            .iter()
            .map(|obj| obj.pretty_print() + "\n")
            .collect::<Vec<String>>()
            .join("")
            .into_bytes();

        file.write_all(&index_content)
            .context("Failed to write content to the index file")?;

        Ok(())
    }

    pub fn clean_index(index_files: &mut Index, deleted_files: Vec<String>) -> Result<&mut Index> {
        index_files.obj.retain(|obj| {
            let file_path = obj.path.to_str().unwrap().to_string();
            if deleted_files.contains(&file_path) {
                eprintln!("Removing deleted file from index: {}", file_path);
                false
            } else {
                true
            }
        });

        Ok(index_files)
    }
}
