use super::error::RGitError;
use anyhow::Result;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_rgit_dir() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;
    loop {
        let rgit_dir = current_dir.join(".rgit");
        if rgit_dir.is_dir() {
            return Ok(rgit_dir);
        }
        if !current_dir.pop() {
            return Err(RGitError::new(
                "fatal: not a rgit repository (or any of the parent directories): .rgit"
                    .to_string(),
                128,
            ));
        }
    }
}

pub fn get_rgit_object_path(hash: &[u8; 20], check_exists: bool) -> Result<PathBuf> {
    let hash = hex::encode(hash);
    let object_path = get_rgit_dir()?.join("objects").join(&hash);
    if check_exists && fs::metadata(&object_path).is_err() {
        return Err(RGitError::new(
            format!("fatal: Not a valid object name {}", hash),
            128,
        ));
    }
    Ok(object_path)
}
