use super::super::error::RGitError;
use super::super::utils::get_rgit_dir;
use clap::Parser;
use sha1::{Digest, Sha1};
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

pub fn rgit_hash_object(args: &HashObjectArgs) -> Result<(), Box<RGitError>> {
    let file = env::current_dir().unwrap().join(&args.file);
    if fs::metadata(&file).is_err() {
        return Err(Box::new(RGitError::new(
            format!(
                "fatal: could not open '{}' for reading: No such file or directory",
                &args.file
            ),
            128,
        )));
    }

    let content = fs::read_to_string(&file).unwrap();
    let size = content.len();
    let data = [
        &b"blob "[..],
        &b" "[..],
        size.to_string().as_bytes(),
        &b"\x00"[..],
        &content.as_bytes(),
    ]
    .concat();
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash_result = hasher.finalize();
    let hash = format!("{:x}", hash_result);

    if args.write {
        let rgit_dir = get_rgit_dir()?;

        let object = rgit_dir.join("objects").join(&hash);
        fs::write(&object, &data).unwrap();
    }

    println!("{}", hash);
    Ok(())
}
