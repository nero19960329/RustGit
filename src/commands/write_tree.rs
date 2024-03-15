use super::super::objects::{RGitObject, Tree};
use super::super::utils::get_rgit_dir;
use anyhow::Result;
use std::env;

pub fn rgit_write_tree() -> Result<u8> {
    let current_dir = env::current_dir()?;
    let rgit_dir = get_rgit_dir(&current_dir)?;

    let tree = Tree::from_path(&current_dir)?;
    let tree_hash = tree.hash()?;
    tree.write_object(rgit_dir.as_path())?;

    println!("{}", hex::encode(tree_hash));
    Ok(0)
}
