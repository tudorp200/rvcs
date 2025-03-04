use crate::commands::diff;
use crate::objects::commit::Commit;
use crate::objects::tree::Tree;
use crate::others::{file_altering, index};
use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
pub fn commit_tree_command(tree_hash: &str, message: &str) -> Result<()> {
    let head_path = Path::new(".vcs").join("HEAD");
    // Determine the parent commit, if it exists
    let parent_commit = if head_path.exists() {
        let branch_ref = fs::read_to_string(&head_path)?.trim().to_string();
        let branch_path = Path::new(".vcs").join(branch_ref);
        if branch_path.exists() {
            let temp = fs::read_to_string(&branch_path)?.trim().to_string();
            if temp.is_empty() {
                None
            } else {
                Some(fs::read_to_string(&branch_path)?.trim().to_string())
            }
        } else {
            None
        }
    } else {
        None
    };
    let parents: Vec<String> = parent_commit.into_iter().collect();
    let commit = Commit::new(tree_hash.to_string(), parents.clone(), message.to_string());
    commit.create_commit()?;

    // Update the current branch reference to point to the new commit
    if head_path.exists() {
        let branch_ref = fs::read_to_string(&head_path)?.trim().to_string();

        let branch_path = Path::new(".vcs").join(branch_ref);

        let mut branch_file =
            fs::File::create(branch_path).map_err(|e| anyhow!("Failed to update branch: {}", e))?;
        branch_file.write_all(commit.id.as_bytes())?;
    } else {
        return Err(anyhow!(
            "HEAD file not found. Is the repository initialized?"
        ));
    }
    /*
     * For better print i will diff between two commits
     *
     */
    println!("Commit created successfully with ID: {}", commit.id);
    if !parents.is_empty() {
        if parents.len() <= 1 {
            return Ok(());
        } else {
            let output = detailed_print(&commit.id, &parents[0])?;

            println!("{}", output);
        }
    }
    Ok(())
}

pub fn get_sections(diff_output: &str) -> Result<Vec<Vec<String>>> {
    let mut sections: Vec<Vec<String>> = Vec::new();
    let mut current_section = Vec::new();
    for line in diff_output.lines() {
        if line.starts_with("@@") && !line.is_empty() {
            sections.push(current_section.clone());
            current_section.clear();
        }
        current_section.push(line.to_string());
    }
    if !current_section.is_empty() {
        sections.push(current_section);
    }
    Ok(sections)
}

pub fn detailed_print(commit_hash: &str, parent_commit: &str) -> Result<String> {
    let mut output = String::new();

    let diff_output = diff::diff_between_commits(parent_commit, commit_hash)?;

    let sections = get_sections(&diff_output)?;
    let mut add_ct: u32 = 0;
    let mut del_ct: u32 = 0;
    let mut mod_ct: u32 = 0;
    let mut map: HashMap<String, (u32, u32)> = HashMap::new();
    let mut file_name = String::new();
    for section in sections.iter() {
        for line in section.iter() {
            if line.starts_with("Added") {
                file_name = line["Added file: ".len()..].to_string();
                map.insert(file_name.clone(), (0, 0));
                add_ct += 1;
            }
            if line.starts_with("Deleted") {
                file_name = line["Deleted file: ".len()..].to_string();
                del_ct += 1;

                map.insert(file_name.clone(), (0, 0));
            }

            if line.starts_with("Modified") {
                file_name = line["Modified file: ".len()..].to_string();
                mod_ct += 1;
                map.insert(file_name.clone(), (0, 0));
            }
            if !file_name.is_empty() {
                if line.starts_with("+") {
                    map.entry(file_name.clone())
                        .and_modify(|e| e.0 += 1)
                        .or_insert((1, 0));
                }
                if line.starts_with("-") {
                    map.entry(file_name.clone())
                        .and_modify(|e| e.1 += 1)
                        .or_insert((0, 1));
                }
            }
        }
    }

    let added_files = format!("{} files added", add_ct);
    let deleted_files = format!("{} files deletes", del_ct);
    let modified_files = format!("{} files modified", mod_ct);
    let temp = format!("{}\n{}\n{}\n", added_files, deleted_files, modified_files);
    output.push_str(&temp);
    for entry in map.iter() {
        let temp = format!(
            "{} file: {} insertions(+), {} deletions(-)",
            entry.0, entry.1 .0, entry.1 .1
        );

        output.push_str(&temp);
    }
    Ok(output)
}

pub fn commit_command(msg: &str) -> Result<()> {
    let current_tree_hash = match file_altering::get_current_tree() {
        Ok(curent_tree_hash) => {
            let (mut current_files, deleted_files) =
                file_altering::get_files_from_tree(&curent_tree_hash)?;

            let size = deleted_files.len();
            let true_index = index::Index::clean_index(&mut current_files, deleted_files)?;
            if size != 0 {
                true_index.save_index_file_truncate()?;
            }
            curent_tree_hash
        }
        Err(_) => {
            eprintln!("first commit");
            String::new()
        }
    };
    let tree = Tree::new()?;
    tree.create_tree()?;
    if !current_tree_hash.is_empty() && current_tree_hash == tree.id {
        return Err(anyhow!("No changes detected. Commit aborted."));
    }
    commit_tree_command(&tree.id, msg)?;
    Ok(())
}

use crate::objects::tree;

pub fn create_merge_commit(commit1: &str, commit2: &str) -> Result<()> {
    let tree1 = file_altering::get_tree_from_commit(commit1)?;
    let tree2 = file_altering::get_tree_from_commit(commit2)?;
    let index1 = tree::Tree::merge_tree(&tree1, &tree2)?;
    let index2 = tree::Tree::merge_tree(&tree1, &tree2)?;
    let merged_tree = tree::Tree::new_tree_from_index(index1)?;
    merged_tree.create_tree_from_index(index2)?;

    let parents: Vec<String> = vec![commit1.to_string(), commit2.to_string()];
    let merged_commit = Commit::new(merged_tree.id, parents, "MERGE COMMIT".to_string());
    merged_commit.create_commit()?;

    println!(
        "Merge commit created successfully with ID: {}",
        merged_commit.id
    );

    let head_path = Path::new(".vcs").join("HEAD");
    let branch_ref = fs::read_to_string(&head_path)?.trim().to_string();
    let branch_path = Path::new(".vcs").join(branch_ref);

    let mut branch_file =
        fs::File::create(branch_path).map_err(|e| anyhow!("Failed to update branch: {}", e))?;
    branch_file.write_all(merged_commit.id.as_bytes())?;

    Ok(())
}
