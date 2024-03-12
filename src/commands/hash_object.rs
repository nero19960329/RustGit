use super::super::objects::{Blob, RGitObject};
use anyhow::Result;
use clap::Parser;
use std::env;
use std::path;

/// Compute object ID
#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    /// Actually write the object into the object database
    #[arg(name = "write", short)]
    pub write: bool,

    pub file: String,
}

fn hash_object(file: &path::Path, write: bool) -> Result<()> {
    let blob = Blob::from_path(&file)?;
    let hash = blob.hash()?;
    if write {
        blob.write_object()?;
    }
    println!("{}", hex::encode(hash));
    Ok(())
}

fn path_from_str(s: &str) -> Result<path::PathBuf> {
    let path = path::PathBuf::from(s);
    if path.is_relative() {
        let current_dir = env::current_dir()?;
        Ok(current_dir.join(path))
    } else {
        Ok(path)
    }
}

pub fn rgit_hash_object(args: &HashObjectArgs) -> Result<()> {
    let file = path_from_str(&args.file)?;
    hash_object(&file, args.write)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_path_from_str() {
        let dir = tempdir().unwrap();
        env::set_current_dir(&dir).unwrap();
        let file = dir.path().join("test.txt");
        fs::File::create(&file).unwrap();
        let path = path_from_str("test.txt").unwrap();
        assert_eq!(path, file);

        let path = path_from_str("/test.txt").unwrap();
        assert_eq!(path, path::PathBuf::from("/test.txt"));
    }
}
