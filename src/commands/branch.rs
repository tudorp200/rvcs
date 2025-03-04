use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;
pub fn branch_command(msg: &str) -> Result<()> {
    let current_branch = fs::read_to_string(".vcs/HEAD")?.to_string();
    let mut content = String::from(".vcs/");
    content.push_str(&current_branch);
    let main_branch_content = fs::read_to_string(content)?;
    let branches_path = Path::new(".vcs").join("refs").join("heads").join(msg);
    let mut new_file = fs::File::create(branches_path)?;
    new_file.write_all(&main_branch_content.into_bytes())?;
    Ok(())
}
