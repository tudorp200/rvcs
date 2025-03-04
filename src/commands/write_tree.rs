use crate::objects::tree::Tree;
use anyhow::Result;

pub fn write_tree_command() -> Result<()> {
    let tree = Tree::new()?;
    tree.create_tree()?;
    Ok(())
}
