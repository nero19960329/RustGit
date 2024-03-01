use super::error::{RGitError, NOT_RGIT_REPOSITORY};
use std::path::PathBuf;

fn find_rgit_dir() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().unwrap();
    loop {
        let rgit_dir = current_dir.join(".rgit");
        if rgit_dir.is_dir() {
            return Some(rgit_dir);
        }
        if !current_dir.pop() {
            return None;
        }
    }
}

pub fn get_rgit_dir() -> Result<PathBuf, RGitError> {
    match find_rgit_dir() {
        Some(dir) => Ok(dir),
        None => Err(RGitError::new(NOT_RGIT_REPOSITORY.to_string(), 128)),
    }
}
