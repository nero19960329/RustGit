use crate::hash::hash_array_from_string;
use crate::ignore::is_ignored;
use crate::objects::Tree;
use crate::utils::get_rgit_dir;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::path;

/// Reads tree information into the index
#[derive(Parser, Debug)]
pub struct ReadTreeArgs {
    /// The tree object to be read.
    pub tree_ish: String,
}

fn empty_dir(dir: &path::Path) -> Result<()> {
    for entry in dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.ends_with(".rgit") {
            continue;
        }
        if is_ignored(&path)?.is_ignored {
            continue;
        }

        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }

    Ok(())
}

fn read_tree(dir: &path::Path, tree_ish: String) -> Result<u8> {
    let rgit_dir = get_rgit_dir(dir)?;
    let tree_hash_array = hash_array_from_string(&tree_ish)?;
    let tree = Tree::from_rgit_objects(rgit_dir.as_path(), &tree_hash_array)?;
    empty_dir(dir)?;
    tree.write_to_directory(dir)?;
    Ok(0)
}

pub fn rgit_read_tree(args: &ReadTreeArgs) -> Result<u8> {
    read_tree(env::current_dir()?.as_path(), args.tree_ish.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use tempfile::tempdir;

    #[test]
    fn test_empty_dir() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        fs::create_dir_all(dir.path().join("test")).unwrap();
        fs::write(dir.path().join("file"), "test").unwrap();
        fs::write(dir.path().join("test").join("file2"), "test").unwrap();

        empty_dir(dir.path()).unwrap();

        // only .rgit should be left
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert!(fs::metadata(rgit_dir).is_ok());
        assert!(fs::metadata(dir.path().join("test")).is_err());
    }

    #[test]
    fn test_read_tree() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let rgit_dir = init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_directory(path).unwrap();
        tree.write_to_rgit_objects(rgit_dir.as_path()).unwrap();

        let result = read_tree(path, hex::encode(tree.hash()));
        assert!(result.is_ok());
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "file content");
        assert_eq!(
            fs::read_to_string(&subfile_path).unwrap(),
            "subfile content"
        );
    }
}
