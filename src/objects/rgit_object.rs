use super::super::utils::get_rgit_object_path;
use super::blob::Blob;
use super::tree::Tree;
use anyhow::{anyhow, Result};
use std::fmt;
use std::fs;
use std::io;
use std::path;

#[derive(Debug, PartialEq)]
pub enum RGitObjectType {
    Tree,
    Blob,
}

impl fmt::Display for RGitObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RGitObjectType::Tree => write!(f, "tree"),
            RGitObjectType::Blob => write!(f, "blob"),
        }
    }
}

pub struct RGitObjectHeader {
    pub object_type: RGitObjectType,
    pub size: usize,
}

impl RGitObjectHeader {
    pub fn new(object_type: RGitObjectType, size: usize) -> Self {
        Self { object_type, size }
    }

    pub fn serialize(&self) -> Vec<u8> {
        format!("{} {}\x00", self.object_type, self.size).into_bytes()
    }

    pub fn deserialize(reader: &mut dyn io::Read) -> Result<Self> {
        let mut header = Vec::new();
        let mut byte = [0; 1];
        loop {
            reader.read_exact(&mut byte)?;
            if byte[0] == b'\x00' {
                break;
            }
            header.push(byte[0]);
        }

        let header = String::from_utf8(header)?;
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(anyhow!("Invalid object header: {:?}", header));
        }

        let object_type = match parts[0] {
            "tree" => RGitObjectType::Tree,
            "blob" => RGitObjectType::Blob,
            _ => return Err(anyhow!("Invalid object type: {:?}", parts[0])),
        };
        let size = parts[1].parse::<usize>()?;

        Ok(Self { object_type, size })
    }
}

pub trait RGitObject {
    fn header(&self) -> Result<RGitObjectHeader>;
    fn hash(&self) -> Result<&[u8; 20]>;
    fn write(&self, rgit_dir: &path::Path, path: &path::Path) -> Result<()>;
    fn write_object(&self, rgit_dir: &path::Path) -> Result<()>;
    fn serialize_object(&self, rgit_dir: &path::Path, writer: &mut dyn io::Write) -> Result<()>;
}

pub fn rgit_object_from_hash(
    rgit_dir: &path::Path,
    hash: &[u8; 20],
) -> Result<Box<dyn RGitObject>> {
    let object_path = get_rgit_object_path(rgit_dir, &hash, true)?;
    let header = RGitObjectHeader::deserialize(&mut fs::File::open(&object_path)?)?;

    match header.object_type {
        RGitObjectType::Tree => Ok(Box::new(Tree::from_hash(rgit_dir, hash.clone())?)),
        RGitObjectType::Blob => Ok(Box::new(Blob::from_hash(rgit_dir, hash.clone())?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use tempfile::tempdir;

    #[test]
    fn test_rgit_object_from_hash() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        fs::write(dir.path().join("file.txt"), "Hello, world!").unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/file.txt"), "Hello, world!").unwrap();

        let tree = Tree::from_path(&dir.path()).unwrap();
        tree.write_object(rgit_dir.as_path()).unwrap();

        let blob = Blob::from_path(&dir.path().join("file.txt")).unwrap();

        let tree = rgit_object_from_hash(rgit_dir.as_path(), tree.hash().unwrap()).unwrap();
        assert_eq!(tree.header().unwrap().object_type, RGitObjectType::Tree);

        let blob = rgit_object_from_hash(rgit_dir.as_path(), blob.hash().unwrap()).unwrap();
        assert_eq!(blob.header().unwrap().object_type, RGitObjectType::Blob);
    }
}
