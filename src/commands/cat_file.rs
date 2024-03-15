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

pub fn rgit_cat_file(args: &CatFileArgs) -> Result<u8> {
    cat_file(
        env::current_dir()?.as_path(),
        args.object.clone(),
        args.t,
        args.s,
        args.p,
        &mut io::stdout(),
    )?;
    Ok(0)
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

        // test under an un-initialized repository
        let mut buffer = Vec::new();
        let result = cat_file(
            dir.path(),
            "invalid_hash".to_string(),
            false,
            false,
            true,
            &mut buffer,
        );
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("fatal: not a rgit repository"));

        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        let file_path = dir.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).unwrap();
        let blob = Blob::from_path(&file_path).unwrap();
        let hash = blob.hash().unwrap();
        blob.write_object(rgit_dir.as_path()).unwrap();

        let mut buffer = Vec::new();
        let result = cat_file(
            dir.path(),
            hex::encode(hash),
            false,
            false,
            true,
            &mut buffer,
        );
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buffer).unwrap(), content);

        let mut buffer = Vec::new();
        let result = cat_file(
            dir.path(),
            hex::encode(hash),
            true,
            false,
            false,
            &mut buffer,
        );
        assert!(result.is_ok());
        assert_eq!(String::from_utf8(buffer).unwrap().trim(), "blob");

        let mut buffer = Vec::new();
        let result = cat_file(
            dir.path(),
            hex::encode(hash),
            false,
            true,
            false,
            &mut buffer,
        );
        assert!(result.is_ok());
        assert_eq!(
            String::from_utf8(buffer).unwrap().trim(),
            content.len().to_string()
        );
    }
}
