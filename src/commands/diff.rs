use crate::commands::dec_object;
use crate::others::file_altering;
use crate::others::index;
use anyhow::Result;
use colored::*;
use std::collections::{HashMap, HashSet};

pub fn diff_between_files(file1: &str, file2: &str) -> Result<String> {
    let content1 = file_altering::get_file_content(file1)?;
    let content2 = file_altering::get_file_content(file2)?;
    let header = format!("{} {} -> {}\n", "@@".blue().bold(), file1, file2);
    println!("{}", header);
    Ok(diff_between_content(&content1, &content2))
}

pub fn diff_between_content(content1: &str, content2: &str) -> String {
    let lines1: Vec<&str> = content1.lines().collect();
    let lines2: Vec<&str> = content2.lines().collect();
    let mut diff_result = String::new();

    let mut i = 0;
    let mut j = 0;

    while i < lines1.len() || j < lines2.len() {
        if i < lines1.len() && j < lines2.len() && lines1[i] == lines2[j] {
            diff_result.push_str(&format!("  {}\n", lines1[i]));
            i += 1;
            j += 1;
        } else if i < lines1.len() && (j >= lines2.len() || lines1[i] != lines2[j]) {
            // Line removed
            diff_result.push_str(&format!("{} {}\n", "-", lines1[i].red()));
            i += 1;
        } else if j < lines2.len() && (i >= lines1.len() || lines1[i] != lines2[j]) {
            // Line added
            diff_result.push_str(&format!("{} {}\n", "+", lines2[j].green()));
            j += 1;
        }
    }

    diff_result
}

pub fn diff_between_obj(obj1: index::Index, obj2: index::Index) -> Result<String> {
    // Create maps for efficient lookup
    let mut content = String::new();
    let obj1_map: HashMap<String, &index::ObjectInfo> = obj1
        .obj
        .iter()
        .map(|obj| (obj.path.to_str().unwrap().to_string(), obj))
        .collect();
    let obj2_map: HashMap<String, &index::ObjectInfo> = obj2
        .obj
        .iter()
        .map(|obj| (obj.path.to_str().unwrap().to_string(), obj))
        .collect();

    let all_files: HashSet<String> = obj1_map
        .keys()
        .cloned()
        .chain(obj2_map.keys().cloned())
        .collect();

    for file in all_files {
        let header = format!("{} {}\n", "@@".blue().bold(), file);
        match (obj1_map.get(&file), obj2_map.get(&file)) {
            (Some(file1), Some(file2)) => {
                if file1.hash != file2.hash {
                    // Modified file
                    content.push_str(&header);
                    content.push('\n');
                    //println!("{}", header);
                    let temp = format!("Modified file: {}", file.blue());
                    //println!("Modified file: {}", file.blue());
                    content.push_str(&temp);
                    content.push('\n');
                    let content1 = dec_object::dec_obj(&file1.hash)?;
                    let content2 = dec_object::dec_obj(&file2.hash)?;
                    let diff = diff_between_content(&content1, &content2);
                    //println!("{}", diff);
                    content.push_str(&diff);
                    content.push('\n');
                }
            }
            (Some(file1), None) => {
                // File deleted

                //println!("{}", header);

                content.push_str(&header);
                content.push('\n');

                let temp = format!("Deleted file: {}", file1.path.to_str().unwrap().red());

                content.push_str(&temp);
                content.push('\n');
                //println!("Deleted file: {}", file1.path.to_str().unwrap().red());
                let content1 = dec_object::dec_obj(&file1.hash)?;
                let diff = diff_between_content(&content1, "");
                //println!("{}", diff);

                content.push_str(&diff);
                content.push('\n');
            }
            (None, Some(file2)) => {
                content.push_str(&header);
                content.push('\n');
                //println!("{}", header);

                let temp = format!("Added file: {}", file2.path.to_str().unwrap().green());
                content.push_str(&temp);
                content.push('\n');
                //println!("Added file: {}", file2.path.to_str().unwrap().green());
                let content1 = dec_object::dec_obj(&file2.hash)?;
                let diff = diff_between_content("", &content1);
                //println!("{}", diff);

                content.push_str(&diff);
                content.push('\n');
            }
            (None, None) => {
                println!("what");
            }
        }
    }

    Ok(content)
}

pub fn diff_between_commits(commit_hash1: &str, commit_hash2: &str) -> Result<String> {
    let tree_hash1 = file_altering::get_tree_from_commit(commit_hash1)?;
    let tree_hash2 = file_altering::get_tree_from_commit(commit_hash2)?;

    let mut branch1 = index::Index::new();
    let tree_content1 = dec_object::dec_obj(&tree_hash1)?;
    for line in tree_content1.lines() {
        let obj = index::ObjectInfo::from_pretty_print(line)?;
        branch1.add_object(obj);
    }

    let mut branch2 = index::Index::new();
    let tree_content2 = dec_object::dec_obj(&tree_hash2)?;
    for line in tree_content2.lines() {
        let obj2 = index::ObjectInfo::from_pretty_print(line)?;
        branch2.add_object(obj2);
    }

    let temp = diff_between_obj(branch1, branch2)?;
    Ok(temp)
}

pub fn diff_between_current_last_commit() -> Result<()> {
    // get current branch commit
    let current_branch_temp = file_altering::get_curent_branch()?;
    let current_branch = current_branch_temp["refs/heads/".len()..].to_string();
    let current_commit_hash = file_altering::get_commit_from_branch(&current_branch)?;
    let previous_commit_hash = file_altering::get_commit_parent(&current_commit_hash)?;
    if previous_commit_hash.len() == 1 {
        let temp = diff_between_commits(&previous_commit_hash[0], &current_commit_hash)?;
        println!("{}", temp);
    } else {
        let temp = diff_between_commits(&previous_commit_hash[1], &current_commit_hash)?;
        println!("{}", temp);
    }
    Ok(())
}

pub fn diff_between_branches(branch1: &str, branch2: &str) -> Result<()> {
    let commit_hash1 = file_altering::get_commit_from_branch(branch1)?;
    let commit_hash2 = file_altering::get_commit_from_branch(branch2)?;
    diff_between_commits(&commit_hash1, &commit_hash2)?;
    Ok(())
}
