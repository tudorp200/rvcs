use crate::commands::dec_object::dec_obj;
use crate::objects::commit;
use crate::others::file_altering;
use anyhow::Result;
use glob::Pattern;
use std::fs;
use std::path::Path;

pub fn parse_gitignore() -> Result<Vec<Pattern>, anyhow::Error> {
    let gitignore_path = ".ignore";

    if !Path::new(gitignore_path).exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(gitignore_path)?;
    let patterns = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .map(|line| {
            let mut pattern = line.trim().to_string();

            if pattern.ends_with('/') {
                pattern = format!("{}/**", pattern.trim_end_matches('/'));
            }

            Pattern::new(&pattern)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(patterns)
}

pub fn status_command() -> Result<String> {
    let current_branch = file_altering::get_curent_branch()?.trim().to_string();
    let mut content = String::new();
    println!("On branch '{}'", current_branch);
    content.push_str(&format!("On branch '{}'", current_branch));
    content.push('\n');
    // Get the current commit and tree
    let mut current_branch_path = String::from(".vcs/");
    current_branch_path.push_str(&file_altering::get_curent_branch()?);
    let current_commit_hash = fs::read_to_string(&current_branch_path)?.trim().to_string();

    let working_files = file_altering::get_working_files()?;
    let mut untracked_files = Vec::new();
    if current_commit_hash.is_empty() {
        for working_file in &working_files.obj {
            untracked_files.push(working_file.path.to_str().unwrap().to_string());
        }
        if untracked_files.is_empty() {
            println!("nothing to commit")
        } else {
            println!("Untracked files:");
            for file in untracked_files.iter() {
                println!("\t{}", file);
            }
        }
        return Ok(String::new());
    }
    let current_pretty_commit = dec_obj(&current_commit_hash)?;
    let current_commit_content_obj =
        commit::CommitContent::from_pretty_print(&current_pretty_commit)?;
    let current_tree = current_commit_content_obj.tree;
    let (index_files, deleted_files) = file_altering::get_files_from_tree(&current_tree)?;

    // get current index files
    // creating a new tree from the actual index
    let new_tree = crate::objects::tree::Tree::new()?;
    new_tree.create_tree()?;
    let (mut new_index, deleted_fil) = file_altering::get_files_from_tree(&new_tree.id)?;
    let mut modified_files = Vec::new();
    let mut staged_files = Vec::new();
    let mut added_files = Vec::new();

    for working_file in &working_files.obj {
        if let Some(index_file) = index_files.obj.iter().find(|index_file| {
            let mut temp = String::from("./");
            let temp1 = index_file.path.to_str().unwrap();
            temp.push_str(temp1);
            temp == working_file.path.to_str().unwrap()
        }) {
            if index_file.hash != working_file.hash {
                modified_files.push(working_file.path.to_str().unwrap().to_string());
            } else {
                staged_files.push(working_file.path.to_str().unwrap().to_string());
            }
        } else {
            untracked_files.push(working_file.path.to_str().unwrap().to_string());
        }
    }

    for file in &new_index.obj {
        if let Some(_index_file) = index_files
            .obj
            .iter()
            .find(|index_file| index_file.path == file.path)
        {
        } else {
            added_files.push(file.path.to_str().unwrap().to_string());
            untracked_files.retain(|ff| {
                let mut temp = String::from("./");
                let temp1 = file.path.to_str().unwrap();
                temp.push_str(temp1);
                temp != *ff
            });
        }
    }

    if modified_files.is_empty()
        && untracked_files.is_empty()
        && deleted_files.is_empty()
        && added_files.is_empty()
    {
        let ttt = String::from("nothing to commit, working tree clean");
        println!("{}", ttt);
        content.push_str(&ttt);
        content.push('\n');
    } else {
        if !modified_files.is_empty() {
            let ttt = String::from("Changes not staged for commit:");
            println!("{}", ttt);
            content.push_str(&ttt);
            content.push('\n');
            for file in modified_files {
                println!("\tmodified: {}", file);

                content.push_str(&format!("\tmodified: {}", file));
                content.push('\n');
            }
        }

        if !deleted_files.is_empty() {
            println!("Changes not staged for commit:");
            for file in deleted_files {
                println!("\tdeleted: {}", file);

                content.push_str(&format!("\tdeleted {}", file));
                content.push('\n');
            }
        }

        if !untracked_files.is_empty() {
            println!("Untracked files:");
            for file in untracked_files {
                println!("\t{}", file);
            }
        }

        if !added_files.is_empty() {
            println!("Added files:");
            for file in added_files {
                println!("\tadded {}", file);

                content.push_str(&format!("\tadded: {}", file));
                content.push('\n');
            }
        }
    }

    // need to update index in case a file is deleted after it is added
    let actual_index = crate::others::index::Index::clean_index(&mut new_index, deleted_fil)?;
    actual_index.save_index_file_truncate()?;
    Ok(content)
}
