use crate::objects::{Blob, RGitObject};
use crate::utils::{get_rgit_dir, get_rgit_object_path};
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::io;
use std::path;

/// Compute object ID
#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    /// Actually write the object into the object database
    #[arg(name = "write", short)]
    pub write: bool,

    pub file: String,
}

fn hash_object(
    dir: &path::Path,
    file: &path::Path,
    write: bool,
    writer: &mut dyn io::Write,
) -> Result<u8> {
    let blob = Blob::from_file(&file)?;
    let hash = blob.hash();
    if write {
        let rgit_object_path = get_rgit_object_path(get_rgit_dir(dir)?.as_path(), hash, false)?;
        fs::create_dir_all(rgit_object_path.parent().unwrap())?;
        blob.serialize(&mut io::BufWriter::new(fs::File::create(
            &rgit_object_path,
        )?))?;
    }
    writeln!(writer, "{}", hex::encode(hash))?;
    Ok(0)
}

pub fn rgit_hash_object(args: &HashObjectArgs) -> Result<u8> {
    hash_object(
        &env::current_dir()?,
        &path::Path::new(&args.file).canonicalize()?,
        args.write,
        &mut io::stdout(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RGitError;
    use crate::utils::init_rgit_dir;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_hash_object() {
        let dir = tempdir().unwrap();

        // test non-existing file
        let mut buffer = Vec::new();
        let result = hash_object(
            dir.path(),
            &path::Path::new("non-existing-file"),
            false,
            &mut buffer,
        );
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .downcast_ref::<RGitError>()
            .unwrap()
            .message
            .contains("fatal: could not open"));

        // test existing file
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "Hello, World!").unwrap();
        let mut buffer = Vec::new();
        let result = hash_object(dir.path(), &file_path, false, &mut buffer);
        assert!(result.is_ok());

        // test write option without initializing the repository
        let mut buffer = Vec::new();
        let result = hash_object(dir.path(), &file_path, true, &mut buffer);
        assert!(result.is_err());
        assert!(result
            .err()
            .unwrap()
            .downcast_ref::<RGitError>()
            .unwrap()
            .message
            .contains("fatal: not a rgit repository"));

        init_rgit_dir(dir.path()).unwrap();
        let mut buffer = Vec::new();
        let result = hash_object(dir.path(), &file_path, true, &mut buffer);
        assert!(result.is_ok());

        let objects_dir = dir.path().join(".rgit/objects");
        assert!(objects_dir.exists());
    }
}
