use crate::error::RGitError;
use crate::hash::hash_object;
use crate::objects::{RGitObject, RGitObjectHeader, RGitObjectType};
use crate::utils::get_rgit_object_path;
use anyhow::Result;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Blob {
    path: PathBuf,
    size: usize,
    content_offset: u64,
    hash: [u8; 20],
}

impl Blob {
    fn new(path: &Path, size: usize, content_offset: u64, hash: Option<[u8; 20]>) -> Result<Self> {
        let hash = if let Some(hash) = hash {
            hash
        } else {
            let mut file = fs::File::open(path)?;
            file.seek(SeekFrom::Start(content_offset))?;
            let mut content = file.take(size as u64);
            hash_object(&mut content)?
        };

        Ok(Self {
            path: path.to_path_buf(),
            size,
            content_offset,
            hash,
        })
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        if fs::metadata(path).is_err() {
            return Err(RGitError::new(
                format!(
                    "fatal: could not open '{}' for reading: No such file or directory",
                    path.to_string_lossy()
                ),
                128,
            ));
        }

        let size = fs::metadata(path)?.len() as usize;
        Self::new(path, size, 0, None)
    }

    pub fn from_rgit_objects(rgit_dir: &Path, hash: &[u8; 20]) -> Result<Self> {
        let object_path = get_rgit_object_path(rgit_dir, hash, true)?;
        let mut reader = fs::File::open(&object_path)?;

        let header = RGitObjectHeader::deserialize(&mut reader)?;
        if header.object_type != RGitObjectType::Blob {
            return Err(anyhow::anyhow!(format!(
                "Invalid object type: {:?}",
                header.object_type
            )));
        }

        let content_offset = reader.stream_position()?;
        Blob::new(
            object_path.as_path(),
            header.content_size,
            content_offset,
            Some(*hash),
        )
    }

    pub fn content(&self) -> Result<impl Read> {
        let mut file = fs::File::open(&self.path)?;
        file.seek(SeekFrom::Start(self.content_offset))?;
        Ok(file.take(self.size as u64))
    }

    pub fn hash(&self) -> &[u8; 20] {
        &self.hash
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let mut src = self.content()?;
        let mut dst = fs::File::create(path)?;
        io::copy(&mut src, &mut dst)?;
        Ok(())
    }

    pub fn write_to_rgit_objects(&self, rgit_dir: &Path) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, &self.hash, false)?;
        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut file = fs::File::create(&object_path)?;
        self.serialize(&mut file)?;
        Ok(())
    }
}

impl RGitObject for Blob {
    fn object_type(&self) -> RGitObjectType {
        RGitObjectType::Blob
    }

    fn size(&self) -> usize {
        self.size
    }

    fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
        let header = RGitObjectHeader::new(self.object_type(), self.size);
        header.serialize(writer)?;
        let mut content = self.content()?;
        io::copy(&mut content, writer)?;
        Ok(())
    }

    fn print(&self, writer: &mut dyn Write) -> Result<()> {
        let mut content = self.content()?;
        io::copy(&mut content, writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Tree;
    use crate::utils::init_rgit_dir;
    use tempfile::tempdir;

    #[test]
    fn test_blob_from_file() {
        let dir = tempdir().unwrap();

        let result = Blob::from_file(dir.path().join("test.txt").as_path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No such file or directory"));

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_file(dir.path().join("test.txt").as_path()).unwrap();
        assert_eq!(blob.size, 13);
    }

    #[test]
    fn test_blob_from_rgit_objects() {
        let dir = tempdir().unwrap();
        init_rgit_dir(dir.path()).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_file(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_to_rgit_objects(dir.path()).unwrap();

        let blob = Blob::from_rgit_objects(dir.path(), &blob.hash).unwrap();
        assert_eq!(blob.size, 13);

        let tree = Tree::from_directory(dir.path()).unwrap();
        tree.write_to_rgit_objects(dir.path()).unwrap();

        let result = Blob::from_rgit_objects(dir.path(), tree.hash());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid object type"));
    }

    #[test]
    fn test_blob_write_to_file() {
        let dir = tempdir().unwrap();
        init_rgit_dir(dir.path()).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_file(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_to_rgit_objects(dir.path()).unwrap();

        fs::remove_file(dir.path().join("test.txt")).unwrap();
        let blob = Blob::from_rgit_objects(dir.path(), &blob.hash).unwrap();
        blob.write_to_file(dir.path().join("test.txt").as_path())
            .unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("test.txt")).unwrap(),
            "Hello, World!"
        );
    }

    #[test]
    fn test_blob_print() {
        let dir = tempdir().unwrap();
        init_rgit_dir(dir.path()).unwrap();

        fs::write(dir.path().join("test.txt"), "Hello, World!").unwrap();
        let blob = Blob::from_file(dir.path().join("test.txt").as_path()).unwrap();
        blob.write_to_rgit_objects(dir.path()).unwrap();

        let mut buffer = Vec::new();
        blob.print(&mut buffer).unwrap();
        assert_eq!(buffer, "Hello, World!".as_bytes(),)
    }
}
