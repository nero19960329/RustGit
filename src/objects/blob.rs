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

        let size = fs::metadata(path)?.len() as usize;
        let header = RGitObjectHeader::new(RGitObjectType::Blob, size);

        Ok(Blob {
            path: Some(path.to_path_buf()),
            hash: hash(vec![header.serialize().as_slice(), &fs::read(path)?].into_iter())?,
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

    fn print_object(&self, rgit_dir: &Path) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, self.hash()?, true)?;
        let mut file = fs::File::open(&object_path)?;
        let header = RGitObjectHeader::deserialize(&mut file)?;
        if header.object_type != RGitObjectType::Blob {
            return Err(anyhow!("Invalid object type: {:?}", header.object_type));
        }
        io::copy(&mut file, &mut io::stdout())?;
        Ok(())
    }
}
