use super::super::hash::hash;
use super::super::ignore::RGitIgnore;
use super::super::utils::get_rgit_object_path;
use super::blob::Blob;
use super::rgit_object::{RGitObject, RGitObjectHeader, RGitObjectType};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path;

#[derive(Debug, PartialEq)]
pub enum EntryType {
    Regular,
    Executable,
    Tree,
    Symlink,
}

impl EntryType {
    pub fn as_str(&self) -> &str {
        match self {
            EntryType::Regular => "100644",
            EntryType::Executable => "100755",
            EntryType::Tree => "040000",
            EntryType::Symlink => "120000",
        }
    }
}

fn get_entry_mode(entry: &fs::DirEntry) -> Result<EntryType> {
    let metadata = entry.metadata()?;
    let file_type = metadata.file_type();
    let permissions = metadata.permissions();
    let mode = permissions.mode();

    if file_type.is_dir() {
        Ok(EntryType::Tree)
    } else if file_type.is_symlink() {
        Ok(EntryType::Symlink)
    } else if mode & 0o111 != 0 {
        Ok(EntryType::Executable)
    } else {
        Ok(EntryType::Regular)
    }
}

pub struct Tree {
    #[allow(dead_code)]
    path: Option<path::PathBuf>,
    hash: String,

    entries: BTreeMap<String, Box<dyn RGitObject>>,
    content: String,
}

impl Tree {
    pub fn from_path(path: &path::PathBuf, rgitignore: &RGitIgnore) -> Result<Self> {
        let mut entries: BTreeMap<String, Box<dyn RGitObject>> = BTreeMap::new();

        let mut content = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if rgitignore.is_ignored(&entry_path, &path)? {
                continue;
            }

            let name = entry.file_name().into_string().unwrap();
            let mode = get_entry_mode(&entry)?;

            let object = match mode {
                EntryType::Tree => {
                    Box::new(Tree::from_path(&entry_path, rgitignore)?) as Box<dyn RGitObject>
                }
                EntryType::Regular | EntryType::Executable => {
                    Box::new(Blob::from_path(&entry_path)?) as Box<dyn RGitObject>
                }
                _ => {
                    return Err(anyhow!("Unsupported entry type: {:?}", mode));
                }
            };

            let line = format!(
                "{} {} {}\t{}\x00",
                mode.as_str(),
                object.header()?.object_type,
                object.hash()?,
                name
            );
            content.extend(line.as_bytes());

            entries.insert(name, object);
        }

        let header = RGitObjectHeader {
            object_type: RGitObjectType::Tree,
            size: content.len(),
        };

        Ok(Self {
            path: Some(path.clone()),
            hash: hash(vec![header.serialize().as_slice(), content.as_slice()].into_iter())?,
            entries: entries,
            content: String::from_utf8_lossy(&content).to_string(),
        })
    }

    pub fn from_hash(hash: String) -> Result<Self> {
        let object_path = get_rgit_object_path(&hash, true)?;
        let mut file = fs::File::open(&object_path)?;
        let header = RGitObjectHeader::deserialize(&mut file)?;
        if header.object_type != RGitObjectType::Tree {
            return Err(anyhow!("Invalid object type: {:?}", header.object_type));
        }

        let mut entries: BTreeMap<String, Box<dyn RGitObject>> = BTreeMap::new();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        for line in buffer.split(|&c| c == b'\x00') {
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&[u8]> = line.split(|&c| c == b' ' || c == b'\t').collect();
            if parts.len() != 4 {
                return Err(anyhow!(
                    "Invalid tree entry: {:?}",
                    String::from_utf8_lossy(line)
                ));
            }

            let object_type = match parts[1] {
                b"blob" => RGitObjectType::Blob,
                b"tree" => RGitObjectType::Tree,
                _ => {
                    return Err(anyhow!(
                        "Invalid object type: {:?}",
                        String::from_utf8_lossy(parts[1])
                    ))
                }
            };
            let hash = String::from_utf8_lossy(parts[2]).to_string();
            let name = String::from_utf8_lossy(parts[3]).to_string();

            let object = match object_type {
                RGitObjectType::Blob => {
                    Box::new(Blob::from_hash(hash.clone())?) as Box<dyn RGitObject>
                }
                RGitObjectType::Tree => {
                    Box::new(Tree::from_hash(hash.clone())?) as Box<dyn RGitObject>
                }
            };
            entries.insert(name, object);
        }

        Ok(Self {
            path: None,
            hash: hash.clone(),
            entries: entries,
            content: String::from_utf8_lossy(&buffer).to_string(),
        })
    }
}

impl RGitObject for Tree {
    fn header(&self) -> Result<RGitObjectHeader> {
        Ok(RGitObjectHeader {
            object_type: RGitObjectType::Tree,
            size: self.content.len(),
        })
    }

    fn hash(&self) -> Result<String> {
        Ok(self.hash.clone())
    }

    fn write(&self) -> Result<()> {
        unimplemented!()
    }

    fn write_object(&self) -> Result<()> {
        let object_path = get_rgit_object_path(&self.hash()?, false)?;

        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut object_file = fs::File::create(&object_path)?;
        object_file.write_all(&self.header()?.serialize())?;
        object_file.write_all(self.content.as_bytes())?;

        for (_, object) in &self.entries {
            object.write_object()?;
        }

        Ok(())
    }

    fn print_object(&self) -> Result<()> {
        let lines: Vec<&str> = self.content.split('\x00').collect();
        for line in lines {
            println!("{}", line);
        }
        Ok(())
    }
}
