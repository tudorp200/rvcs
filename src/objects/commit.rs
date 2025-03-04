use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::others::compression;
use crate::others::hash_function::calculate_hash;

pub struct Commit {
    pub id: String, // commit hash_function
    pub content: CommitContent,
}

pub struct CommitContent {
    pub tree: String,         // SHA1 of the tree object
    pub parents: Vec<String>, // SHA1(s) of parent commits
    pub message: String,      // Commit message
}

impl CommitContent {
    pub fn new(tree: String, parents: Vec<String>, message: String) -> Self {
        CommitContent {
            tree,
            parents,
            message,
        }
    }
    pub fn pretty_print(&self) -> String {
        let mut commit_data = format!("tree {}\n", self.tree);

        for parent in &self.parents {
            commit_data.push_str(&format!("parent {}\n", parent));
        }

        commit_data.push_str(&format!("\n{}\n", self.message));

        commit_data
    }
    pub fn from_pretty_print(content: &str) -> Result<Self> {
        let lines = content.lines();
        let mut tree = String::new();
        let mut parents = Vec::new();
        let mut message = String::new();
        let mut in_message = false;

        for line in lines {
            if in_message {
                if !message.is_empty() {
                    message.push('\n');
                }
                message.push_str(line);
            } else if let Some(stripped) = line.strip_prefix("tree ") {
                tree = stripped.to_string();
            } else if let Some(stripped) = line.strip_prefix("parent ") {
                parents.push(stripped.to_string());
            } else if line.trim().is_empty() {
                in_message = true;
            } else {
                return Err(anyhow::anyhow!("Unexpected line in commit: {}", line));
            }
        }

        if tree.is_empty() {
            return Err(anyhow::anyhow!("Missing tree hash in commit content."));
        }

        if message.is_empty() {
            return Err(anyhow::anyhow!("Missing commit message."));
        }

        Ok(Self {
            tree,
            parents,
            message,
        })
    }
}

impl Commit {
    pub fn new(tree: String, parents: Vec<String>, message: String) -> Self {
        let content = CommitContent::new(tree, parents, message);
        let content_as_bytes = content.pretty_print().into_bytes();
        let id = calculate_hash(&content_as_bytes);
        Self { id, content }
    }
    pub fn create_commit(&self) -> Result<()> {
        let vcs_objects_path: PathBuf = Path::new(".vcs").join("objects");

        if !vcs_objects_path.exists() {
            return Err(anyhow::anyhow!("The repository is not initialized"));
        }

        // Serialize commit object
        let commit_content = self.content.pretty_print();
        let header = format!("commit{}\0", commit_content.len());
        let mut content = header.into_bytes();
        content.extend_from_slice(commit_content.as_bytes());
        let compressed_content = compression::compress(commit_content.as_bytes())?;

        // Create the object subfolder
        let subfolder = vcs_objects_path.join(&self.id[0..2]);
        if !subfolder.exists() {
            fs::create_dir_all(&subfolder)?;
        }

        // Write the commit object to the file
        let file_path = subfolder.join(&self.id[2..]);
        let mut file = fs::File::create(file_path)?;
        file.write_all(&compressed_content)?;

        Ok(())
    }
}
