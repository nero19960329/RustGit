use super::super::error::RGitError;
use super::super::hash::hash;
use super::super::utils::get_rgit_object_path;
use super::rgit_object::{RGitObject, RGitObjectHeader, RGitObjectType};
use anyhow::{anyhow, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Blob {
    path: Option<PathBuf>,
    hash: [u8; 20],

    size: usize,
}

impl Blob {
    pub fn from_path(path: &Path) -> Result<Blob> {
        if fs::metadata(path).is_err() {
            return Err(RGitError::new(
                format!(
                    "fatal: could not open '{}' for reading: No such file or directory",
                    path.to_string_lossy()
                ),
                128,
            ));
        }

        Ok(Blob {
            path: Some(path.to_path_buf()),
            hash: hash(vec![fs::File::open(path)?].into_iter())?,
            size: fs::metadata(&path)?.len() as usize,
        })
    }

    pub fn from_hash(rgit_dir: &Path, hash: [u8; 20]) -> Result<Blob> {
        let object_path = get_rgit_object_path(rgit_dir, &hash, true)?;
        let header = RGitObjectHeader::deserialize(&mut fs::File::open(&object_path)?)?;
        if header.object_type != RGitObjectType::Blob {
            return Err(anyhow!("Invalid object type: {:?}", header.object_type));
        }
        Ok(Blob {
            path: None,
            hash: hash,
            size: header.size,
        })
    }
}

impl RGitObject for Blob {
    fn header(&self) -> Result<RGitObjectHeader> {
        Ok(RGitObjectHeader::new(RGitObjectType::Blob, self.size))
    }

    fn hash(&self) -> Result<&[u8; 20]> {
        Ok(&self.hash)
    }

    fn write(&self, rgit_dir: &Path) -> Result<()> {
        let path = self.path.as_ref().unwrap();
        let object_path = get_rgit_object_path(rgit_dir, self.hash()?, true)?;

        fs::create_dir_all(path.parent().unwrap())?;
        let mut file = fs::File::create(path)?;
        let mut object_file = fs::File::open(&object_path)?;
        RGitObjectHeader::deserialize(&mut object_file)?;
        io::copy(&mut object_file, &mut file)?;

        Ok(())
    }

    fn write_object(&self, rgit_dir: &Path) -> Result<()> {
        let path = self.path.as_ref().unwrap();
        let object_path = get_rgit_object_path(rgit_dir, self.hash()?, false)?;

        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut file = fs::File::open(path)?;
        let mut object_file = fs::File::create(&object_path)?;
        object_file.write_all(&self.header()?.serialize())?;
        io::copy(&mut file, &mut object_file)?;

        Ok(())
    }

    fn serialize_object(&self, rgit_dir: &Path, writer: &mut dyn Write) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, self.hash()?, true)?;
        let mut file = fs::File::open(&object_path)?;
        RGitObjectHeader::deserialize(&mut file)?;
        io::copy(&mut file, writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Tree;
    use tempfile::tempdir;

    #[test]
    fn test_blob_from_path() {
        let dir = tempdir().unwrap();

        let result = Blob::from_path(dir.path().join("test.txt").as_path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No such file or directory"));

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_path(dir.path().join("test.txt").as_path()).unwrap();
        assert_eq!(blob.size, 13);
    }

    #[test]
    fn test_blob_from_hash() {
        let dir = tempdir().unwrap();
        let rgit_dir = dir.path().join(".rgit");
        fs::create_dir_all(rgit_dir).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_path(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_object(dir.path()).unwrap();

        let blob = Blob::from_hash(dir.path(), *blob.hash().unwrap()).unwrap();
        assert_eq!(blob.size, 13);

        let tree = Tree::from_path(dir.path()).unwrap();
        tree.write_object(dir.path()).unwrap();

        let result = Blob::from_hash(dir.path(), *tree.hash().unwrap());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid object type"));
    }

    #[test]
    fn test_blob_write() {
        let dir = tempdir().unwrap();
        let rgit_dir = dir.path().join(".rgit");
        fs::create_dir_all(rgit_dir).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_path(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_object(dir.path()).unwrap();

        fs::remove_file(dir.path().join("test.txt")).unwrap();
        blob.write(dir.path()).unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("test.txt")).unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn test_blob_serialize() {
        let dir = tempdir().unwrap();
        let rgit_dir = dir.path().join(".rgit");
        fs::create_dir_all(rgit_dir).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_path(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_object(dir.path()).unwrap();

        let mut buffer = Vec::new();
        blob.serialize_object(dir.path(), &mut buffer).unwrap();
        assert_eq!(buffer, "Hello, World!".as_bytes(),)
    }
}
