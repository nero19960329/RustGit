use super::error::RGitError;
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_rgit_dir(dir: &Path) -> Result<PathBuf> {
    let mut dir = dir.to_path_buf();
    if dir.is_file() {
        dir.pop();
    }

    loop {
        let rgit_dir = dir.join(".rgit");
        if rgit_dir.is_dir() {
            return Ok(rgit_dir);
        }
        if !dir.pop() {
            return Err(RGitError::new(
                "fatal: not a rgit repository (or any of the parent directories): .rgit"
                    .to_string(),
                128,
            ));
        }
    }
}

pub fn get_rgit_object_path(
    rgit_dir: &Path,
    hash: &[u8; 20],
    check_exists: bool,
) -> Result<PathBuf> {
    if fs::metadata(rgit_dir).is_err() || rgit_dir.is_file() {
        return Err(RGitError::new(
            "fatal: not a rgit repository (or any of the parent directories): .rgit".to_string(),
            128,
        ));
    }

    let hash = hex::encode(hash);
    let object_path = rgit_dir.join("objects").join(&hash[..2]).join(&hash[2..]);
    if check_exists && fs::metadata(&object_path).is_err() {
        return Err(RGitError::new(
            format!("fatal: Not a valid object name {}", hash),
            128,
        ));
    }
    Ok(object_path)
}
