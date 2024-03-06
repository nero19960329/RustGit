use super::error::RGitError;
use anyhow::Result;
use std::env;
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
