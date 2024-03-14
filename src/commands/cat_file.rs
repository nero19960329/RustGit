use super::super::error::RGitError;
use super::super::objects::rgit_object_from_hash;
use super::super::utils::get_rgit_dir;
use anyhow::Result;
use clap::{ArgGroup, Parser};
use std::env;
use std::io;
use std::path;

/// Provide content for repository objects
#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("mode").required(true).args(&["t", "s", "p"])))]
pub struct CatFileArgs {
    /// Instead of the content, show the object type identified by <object>
    #[arg(name = "t", short)]
    pub t: bool,

    /// Instead of the content, show the object size identified by <object>
    #[arg(short)]
    pub s: bool,

    /// Pretty-print the contents of <object> based on its type
    #[arg(short)]
    pub p: bool,

    /// The name of the object to show
    pub object: String,
}

fn cat_file(
    dir: &path::Path,
    object: String,
    t: bool,
    s: bool,
    p: bool,
    writer: &mut dyn io::Write,
) -> Result<()> {
    let rgit_dir = get_rgit_dir(dir)?;
    let mut hash_array = [0; 20];
    hex::decode_to_slice(&object, &mut hash_array)
        .map_err(|_| RGitError::new(format!("fatal: Not a valid object name {}", object), 128))?;

    let rgit_object = rgit_object_from_hash(rgit_dir.as_path(), &hash_array)?;
    if t {
        writeln!(writer, "{}", rgit_object.header()?.object_type)?;
    } else if s {
        writeln!(writer, "{}", rgit_object.header()?.size)?;
    } else if p {
        rgit_object.serialize_object(rgit_dir.as_path(), writer)?;
    }

    Ok(())
}

pub fn rgit_cat_file(args: &CatFileArgs) -> Result<()> {
    cat_file(
        env::current_dir()?.as_path(),
        args.object.clone(),
        args.t,
        args.s,
        args.p,
        &mut io::stdout(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::{Blob, RGitObject};
    use crate::utils::init_rgit_dir;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_cat_file() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        fs::write(rgit_dir.join("test.txt"), "Hello, World!").unwrap();

        let blob = Blob::from_path(rgit_dir.join("test.txt").as_path()).unwrap();
        blob.write_object(rgit_dir.as_path()).unwrap();
        let hash = blob.hash().unwrap();

        let mut buffer = Vec::new();
        cat_file(
            rgit_dir.as_path(),
            hex::encode(hash),
            false,
            false,
            true,
            &mut buffer,
        )
        .unwrap();

        assert_eq!(buffer, b"Hello, World!");
    }
}
