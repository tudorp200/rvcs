pub mod index;

pub mod compression {
    use anyhow::{Context, Result};
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::{Read, Write};

    pub fn compress(content: &[u8]) -> Result<Vec<u8>> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(content)
            .context("Failed to compress content")?;
        encoder.finish().context("Failed to finish compression")
    }

    pub fn decompress(content: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::ZlibDecoder;
        let mut decoder = ZlibDecoder::new(content);
        let mut decompressed_content = Vec::new();
        decoder
            .read_to_end(&mut decompressed_content)
            .context("Failed to decompress content")?;
        Ok(decompressed_content)
    }
}

pub mod hash_function {

    pub fn calculate_hash(content: &[u8]) -> String {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }
}

pub mod file_altering {
    use crate::commands::dec_object::dec_obj;
    use crate::commands::status::parse_gitignore;
    use crate::objects::commit;
    use crate::others::index;
    use anyhow::{Context, Result};
    use glob::Pattern;
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::path::Path;
    pub fn delete_nth_line(line_number: usize, path: &str) -> Result<()> {
        let file_path = Path::new(path);
        let file = fs::File::open(file_path).context("Failed to open file")?;
        let reader = BufReader::new(file);

        let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

        if line_number >= lines.len() {
            return Err(anyhow::anyhow!(
                "Invalid line number: {}. The file has only {} lines.",
                line_number,
                lines.len()
            ));
        }

        lines.remove(line_number);

        let mut file = fs::File::create(file_path).context("Failed to create file")?;
        for line in lines {
            writeln!(file, "{}", line).context("Failed to write to file")?;
        }

        Ok(())
    }

    pub fn get_all_filenames(directory: &str, ignore_patt: &[Pattern]) -> Result<String> {
        let path = Path::new(directory);

        if !path.is_dir() {
            return Err(anyhow::anyhow!(
                "The provided path is not a directory: {}",
                directory
            ));
        }

        let mut filenames = String::new();
        collect_filenames(path, &mut filenames, ignore_patt)?;

        Ok(filenames)
    }

    fn collect_filenames(
        path: &Path,
        filenames: &mut String,
        ignore_patt: &[Pattern],
    ) -> Result<()> {
        for entry in std::fs::read_dir(path).context("Failed to read directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if let Ok(relative_path) = path.strip_prefix(".") {
                let relative_str = relative_path.to_string_lossy();
                if ignore_patt
                    .iter()
                    .any(|pattern| pattern.matches(&relative_str))
                {
                    continue;
                }
            }

            if path.is_dir() {
                collect_filenames(&path, filenames, ignore_patt)?;
            } else if let Some(name) = path.to_str() {
                filenames.push_str(name);
                filenames.push('\n');
            } else {
                return Err(anyhow::anyhow!("Failed to convert file path to string"));
            }
        }
        Ok(())
    }

    pub fn get_curent_branch() -> Result<String> {
        let head_content = fs::read_to_string(".vcs/HEAD")?;
        Ok(head_content)
    }
    pub fn get_files_from_tree(tree_hash: &str) -> Result<(index::Index, Vec<String>)> {
        let tree_content = dec_obj(tree_hash)?;
        let mut index_tree = index::Index::new();
        let mut deleted_files = Vec::new();
        for line in tree_content.lines() {
            // Parse the object from the tree content
            let parsed_object = index::ObjectInfo::from_pretty_print(line)?;

            if parsed_object.obj_type == "blob" {
                // Safely create the ObjectInfo, handling errors
                match index::ObjectInfo::new("blob", &parsed_object.path, &parsed_object.hash) {
                    Ok(obj_info) => index_tree.add_object(obj_info),
                    Err(_) => {
                        deleted_files.push(parsed_object.path.to_str().unwrap().to_string());
                    }
                }
            }
        }

        Ok((index_tree, deleted_files))
    }

    pub fn get_working_files() -> Result<index::Index> {
        let mut working_index = index::Index::new();
        let ignore_patt = parse_gitignore()?;
        let filenames = get_all_filenames(".", &ignore_patt)?;
        for file_path in filenames.lines() {
            if !file_path.starts_with(".vcs") {
                let content = fs::read(file_path)?;
                let file_hash = crate::others::hash_function::calculate_hash(&content);
                let path = Path::new(file_path);
                //let path = Path::new(&file_path[2..]);
                let obj_info = index::ObjectInfo::new("blob", path, &file_hash);
                working_index.add_object(obj_info?);
            }
        }
        Ok(working_index)
    }
    pub fn get_tree_from_commit(commit_hash: &str) -> Result<String> {
        let commit_content = dec_obj(commit_hash)?;
        let commit_obj = commit::CommitContent::from_pretty_print(&commit_content)?;
        Ok(commit_obj.tree)
    }
    pub fn get_current_tree() -> Result<String> {
        let mut current_branch_path = String::from(".vcs/");
        current_branch_path.push_str(&get_curent_branch()?);
        let current_commit_hash = fs::read_to_string(&current_branch_path);
        let mut current_commit_hash1 = String::new();

        match current_commit_hash {
            Ok(ans) => {
                if ans.is_empty() {
                } else {
                    current_commit_hash1.push_str(&ans);
                }
            }
            Err(e) => {
                return Err(e.into());
            }
        }
        if !current_commit_hash1.is_empty() {
            let current_pretty_commit = dec_obj(&current_commit_hash1)?;
            let current_commit_content_obj =
                commit::CommitContent::from_pretty_print(&current_pretty_commit)?;
            Ok(current_commit_content_obj.tree)
        } else {
            Err(anyhow::anyhow!("err"))
        }
    }

    pub fn get_file_content(file: &str) -> Result<String> {
        let content = fs::read_to_string(file)?;
        Ok(content)
    }
    /*pub fn get_current_commit() -> Result<String> {
        let mut current_branch_path = String::from(".vcs/");
        current_branch_path.push_str(&get_curent_branch()?);
        let current_commit_hash = fs::read_to_string(&current_branch_path)?.trim().to_string();
        Ok(current_commit_hash)
    }*/
    pub fn get_commit_parent(commit_hash: &str) -> Result<Vec<String>> {
        let commit_content = dec_obj(commit_hash)?;
        let commit_obj = commit::CommitContent::from_pretty_print(&commit_content)?;
        /*if commit_obj.parents.is_empty() {
            return Err(anyhow::anyhow!("There is only one commit"));
        }*/
        Ok(commit_obj.parents.clone())
    }

    pub fn get_commit_from_branch(branch: &str) -> Result<String> {
        let path = Path::new(".vcs").join("refs").join("heads/");
        let mut branch_path = path.to_str().unwrap().to_string();
        branch_path.push_str(branch);
        let content = fs::read_to_string(branch_path)?;
        Ok(content)
    }

    pub fn build_index_from_tree(tree_hash: &str) -> Result<index::Index> {
        let content = dec_obj(tree_hash)?;

        let mut index = index::Index::new();
        for line in content.lines() {
            let obj_info = index::ObjectInfo::from_pretty_print(line)?;
            index.add_object(obj_info);
        }
        Ok(index)
    }
}
