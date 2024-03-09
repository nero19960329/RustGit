use super::super::ignore::{load_ignore_rules, RGitIgnore};
use super::super::objects::{RGitObject, Tree};
use anyhow::Result;
use std::env;

pub fn rgit_write_tree() -> Result<()> {
    let current_dir = env::current_dir()?;
    let ignore_files = RGitIgnore::load_ignore_files(&current_dir);
    let rgitignore = load_ignore_rules(&ignore_files)?;

    let tree = Tree::from_path(&current_dir, &rgitignore)?;
    let tree_hash = tree.hash()?;
    tree.write_object()?;

    println!("{}", hex::encode(tree_hash));
    Ok(())
}
