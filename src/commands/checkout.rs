use crate::commands::dec_object::dec_obj;
use crate::objects::commit;
use crate::others::file_altering;
use crate::others::index;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn move_head_pointer(branch_name: &str) -> Result<()> {
    let head_path = Path::new(".vcs").join("HEAD");
    let mut head_file = fs::File::create(head_path)?;
    let mut content = String::from("refs/heads/");
    content.push_str(branch_name);
    head_file.write_all(&content.into_bytes())?;
    Ok(())
}

pub fn get_files_from_tree(tree_hash: &str) -> Result<HashMap<String, String>> {
    let tree_content = dec_obj(tree_hash)?;
    let mut file_map = HashMap::new();
    for line in tree_content.lines() {
        let parsed_object = index::ObjectInfo::from_pretty_print(line)?;
        if parsed_object.obj_type == "blob" {
            let path = parsed_object.path.to_str().unwrap();
            file_map.insert(path.to_string(), parsed_object.hash.clone());
        }
    }
    Ok(file_map)
}

pub fn checkout(branch: &str) -> Result<()> {
    let head_path = Path::new(".vcs").join("HEAD");

    let mut current_branch = String::from(".vcs/");
    let content = fs::read_to_string(&head_path)?.trim().to_string();
    current_branch.push_str(&content);
    let current_commit_hash = fs::read_to_string(&current_branch)?.trim().to_string();
    let current_pretty_commit = dec_obj(&current_commit_hash)?;
    let current_commit_content_obj =
        commit::CommitContent::from_pretty_print(&current_pretty_commit)?;
    let current_tree = current_commit_content_obj.tree;

    let target_branch_path = Path::new(".vcs").join("refs").join("heads").join(branch);
    if !target_branch_path.exists() {
        return Err(anyhow!("Branch '{}' does not exist.", branch));
    }
    let target_commit_hash = fs::read_to_string(&target_branch_path)?.trim().to_string();
    let pretty_commit = dec_obj(&target_commit_hash)?;
    let commit_content_obj = commit::CommitContent::from_pretty_print(&pretty_commit)?;
    let target_tree = commit_content_obj.tree;
    let current_files = get_files_from_tree(&current_tree)?;
    let target_files = get_files_from_tree(&target_tree)?;
    let target_index = file_altering::build_index_from_tree(&target_tree)?;
    for file in current_files.keys() {
        if !target_files.contains_key(file) {
            fs::remove_file(file)?;
        }
    }

    for (file, hash) in &target_files {
        let content = dec_obj(hash)?;
        fs::write(file, content)?;
    }

    move_head_pointer(branch)?;
    target_index.save_index_file_truncate()?;
    println!("Switched to branch '{}'.", branch);
    Ok(())
}
