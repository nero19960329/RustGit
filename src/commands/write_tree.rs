use super::super::objects::{RGitObject, Tree};
use anyhow::Result;
use std::env;

pub fn rgit_write_tree() -> Result<()> {
    let current_dir = env::current_dir()?;

    let tree = Tree::from_path(&current_dir)?;
    let tree_hash = tree.hash()?;
    tree.write_object()?;

    println!("{}", hex::encode(tree_hash));
    Ok(())
}
