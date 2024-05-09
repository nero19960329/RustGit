use crate::objects::{Blob, Tree};
use crate::utils::get_rgit_object_path;
use anyhow::Result;
use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum RGitObjectType {
    Blob,
    Tree,
}

impl fmt::Display for RGitObjectType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RGitObjectType::Blob => write!(f, "blob"),
            RGitObjectType::Tree => write!(f, "tree"),
        }
    }
}

impl RGitObjectType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "blob" => Ok(RGitObjectType::Blob),
            "tree" => Ok(RGitObjectType::Tree),
            _ => Err(anyhow::anyhow!("Invalid object type: {:?}", s)),
        }
    }
}

#[derive(Debug)]
pub struct RGitObjectHeader {
    pub object_type: RGitObjectType,
    pub content_size: usize,
}

impl RGitObjectHeader {
    pub fn new(object_type: RGitObjectType, content_size: usize) -> Self {
        Self {
            object_type,
            content_size,
        }
    }

    pub fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
        writer.write_all(format!("{} {}\0", self.object_type, self.content_size).as_bytes())?;
        Ok(())
    }

    pub fn deserialize(reader: &mut dyn Read) -> Result<Self> {
        let mut header = Vec::new();
        let mut byte = [0; 1];
        loop {
            reader.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            header.push(byte[0]);
        }

        let header = String::from_utf8(header)?;

        let mut header_parts = header.split_whitespace();
        let object_type = header_parts
            .next()
            .ok_or(anyhow::anyhow!("Invalid object header: {:?}", header))?
            .to_string();
        let content_size = header_parts
            .next()
            .ok_or(anyhow::anyhow!("Invalid object header: {:?}", header))?
            .parse::<usize>()?;

        let object_type = RGitObjectType::from_str(&object_type)?;

        Ok(Self {
            object_type,
            content_size,
        })
    }
}

pub trait RGitObject {
    fn object_type(&self) -> RGitObjectType;
    fn size(&self) -> usize;

    fn serialize(&self, writer: &mut dyn Write) -> Result<()>;
    fn print(&self, writer: &mut dyn Write) -> Result<()>;
}

pub fn from_rgit_objects(rgit_dir: &Path, hash: &[u8; 20]) -> Result<Box<dyn RGitObject>> {
    let object_path = get_rgit_object_path(rgit_dir, &hash, true)?;
    let mut reader = fs::File::open(&object_path)?;

    let header = RGitObjectHeader::deserialize(&mut reader)?;
    match header.object_type {
        RGitObjectType::Blob => Ok(Box::new(Blob::from_rgit_objects(rgit_dir, hash)?)),
        RGitObjectType::Tree => Ok(Box::new(Tree::from_rgit_objects(rgit_dir, hash)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use tempfile::tempdir;

    #[test]
    fn test_from_rgit_objects() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        fs::write(dir.path().join("file.txt"), "Hello, world!").unwrap();
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("subdir/file.txt"), "Hello, world!").unwrap();

        let tree = Tree::from_directory(&dir.path()).unwrap();
        tree.write_to_rgit_objects(rgit_dir.as_path()).unwrap();

        let blob = Blob::from_file(&dir.path().join("file.txt")).unwrap();

        let tree = from_rgit_objects(rgit_dir.as_path(), tree.hash()).unwrap();
        assert_eq!(tree.object_type(), RGitObjectType::Tree);

        let blob = from_rgit_objects(rgit_dir.as_path(), blob.hash()).unwrap();
        assert_eq!(blob.object_type(), RGitObjectType::Blob);
    }
}
