use anyhow::Result;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn initialize_repo() -> Result<()> {
    let vcs_dir = Path::new(".vcs");
    let objects_dir = vcs_dir.join("objects");
    let refs_dir = vcs_dir.join("refs");
    let heads_dir = refs_dir.join("heads");
    let index_file = vcs_dir.join("index");
    let head_file = vcs_dir.join("HEAD");
    if vcs_dir.exists() {
        println!("Repository already initialized!");
        return Ok(());
    }
    fs::create_dir_all(&objects_dir)?;
    fs::create_dir_all(&refs_dir)?;
    fs::create_dir_all(&heads_dir)?;
    fs::File::create(&index_file)?;
    let mut head = fs::File::create(&head_file)?;
    let main_file_path = heads_dir.join("main");
    let _main_file = fs::File::create(&main_file_path)?;
    head.write_all(b"refs/heads/main")?;
    Ok(())
}
