use super::super::objects::{Blob, RGitObject};
use super::super::utils::get_rgit_dir;
use anyhow::Result;
use clap::Parser;
use std::env;

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
    let blob = Blob::from_path(&file)?;
    let hash = blob.hash()?;
    if args.write {
        let rgit_dir = get_rgit_dir(env::current_dir()?.as_path())?;
        blob.write_object(rgit_dir.as_path())?;
    }
    println!("{}", hex::encode(hash));
    Ok(())
}
