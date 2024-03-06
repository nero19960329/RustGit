use super::super::error::RGitError;
use super::super::hash::hash_object;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;

/// Compute object ID
#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    /// Actually write the object into the object database
    #[arg(name = "write", short)]
    pub write: bool,

    pub file: String,
}

pub fn rgit_hash_object(args: &HashObjectArgs) -> Result<()> {
    let file = env::current_dir()?.join(&args.file);
    if fs::metadata(&file).is_err() {
        return Err(RGitError::new(
            format!(
                "fatal: could not open '{}' for reading: No such file or directory",
                &args.file
            ),
            128,
        ));
    }

    let content = fs::read(&file)?;
    let hash = hash_object(&content, "blob", args.write)?;

    println!("{}", hash);
    Ok(())
}
