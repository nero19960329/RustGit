use super::super::hash::hash;
use super::super::ignore::is_ignored;
use super::super::utils::get_rgit_object_path;
use super::blob::Blob;
use super::rgit_object::{RGitObject, RGitObjectHeader, RGitObjectType};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "100644" => Ok(EntryType::Regular),
            "100755" => Ok(EntryType::Executable),
            "040000" => Ok(EntryType::Tree),
            "120000" => Ok(EntryType::Symlink),
            _ => Err(anyhow!("Invalid entry type: {}", s)),
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

struct TreeEntry {
    pub entry_type: EntryType,
    pub rgit_object: Box<dyn RGitObject>,
}

pub struct Tree {
    #[allow(dead_code)]
    path: Option<PathBuf>,
    hash: [u8; 20],

    entries: BTreeMap<String, TreeEntry>,
    content: Vec<u8>,
}

impl Tree {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut entries: BTreeMap<String, TreeEntry> = BTreeMap::new();

        let mut content: Vec<u8> = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();
            if is_ignored(&entry_path)?.is_ignored {
                continue;
            };

            let name = entry.file_name().into_string().unwrap();
            let mode = get_entry_mode(&entry)?;

            let object = match mode {
                EntryType::Tree => Box::new(Tree::from_path(&entry_path)?) as Box<dyn RGitObject>,
                EntryType::Regular | EntryType::Executable => {
                    Box::new(Blob::from_path(&entry_path)?) as Box<dyn RGitObject>
                }
                _ => {
                    return Err(anyhow!("Unsupported entry type: {:?}", mode));
                }
            };

            content.extend(format!("{} {}\x00", mode.as_str(), name).as_bytes());
            content.extend(object.hash()?);

            entries.insert(
                name,
                TreeEntry {
                    entry_type: mode,
                    rgit_object: object,
                },
            );
        }

        let header = RGitObjectHeader {
            object_type: RGitObjectType::Tree,
            size: content.len(),
        };

        Ok(Self {
            path: Some(path.to_path_buf()),
            hash: hash(vec![header.serialize().as_slice(), content.as_slice()].into_iter())?,
            entries: entries,
            content: content,
        })
    }

    pub fn from_hash(hash: [u8; 20]) -> Result<Self> {
        let object_path = get_rgit_object_path(&hash, true)?;
        let mut file = fs::File::open(&object_path)?;
        let header = RGitObjectHeader::deserialize(&mut file)?;
        if header.object_type != RGitObjectType::Tree {
            return Err(anyhow!("Invalid object type: {:?}", header.object_type));
        }

        let mut entries: BTreeMap<String, TreeEntry> = BTreeMap::new();

        let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();
        let mut bytes = Vec::new();
        loop {
            reader.read_until(b' ', &mut bytes)?;
            if bytes.is_empty() {
                break;
            }
            buffer.extend(&bytes);

            let mode = String::from_utf8_lossy(&bytes[..bytes.len() - 1]).to_string();
            let mode = EntryType::from_str(&mode)?;
            bytes.clear();

            reader.read_until(b'\x00', &mut bytes)?;
            buffer.extend(&bytes);
            let name = String::from_utf8_lossy(&bytes[..bytes.len() - 1]).to_string();
            bytes.clear();

            let mut hash = [0; 20];
            reader.read_exact(&mut hash)?;
            buffer.extend(&hash);

            let object = match mode {
                EntryType::Tree => Box::new(Tree::from_hash(hash)?) as Box<dyn RGitObject>,
                EntryType::Regular | EntryType::Executable => {
                    Box::new(Blob::from_hash(hash)?) as Box<dyn RGitObject>
                }
                _ => {
                    return Err(anyhow!("Unsupported entry type: {:?}", mode));
                }
            };
            entries.insert(
                name,
                TreeEntry {
                    entry_type: mode,
                    rgit_object: object,
                },
            );
        }

        Ok(Self {
            path: None,
            hash: hash.clone(),
            entries: entries,
            content: buffer,
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

    fn hash(&self) -> Result<&[u8; 20]> {
        Ok(&self.hash)
    }

    fn write(&self) -> Result<()> {
        unimplemented!()
    }

    fn write_object(&self) -> Result<()> {
        let object_path = get_rgit_object_path(self.hash()?, false)?;

        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut object_file = fs::File::create(&object_path)?;
        object_file.write_all(&self.header()?.serialize())?;
        object_file.write_all(&self.content)?;

        for (_, tree_entry) in &self.entries {
            tree_entry.rgit_object.write_object()?;
        }

        Ok(())
    }

    fn print_object(&self) -> Result<()> {
        for (name, tree_entry) in &self.entries {
            let rgit_object_type = match tree_entry.rgit_object.header()? {
                RGitObjectHeader { object_type, .. } => object_type,
            };
            let rgit_object_hash = tree_entry.rgit_object.hash()?;

            println!(
                "{} {} {}\t{}",
                tree_entry.entry_type.as_str(),
                rgit_object_type,
                hex::encode(rgit_object_hash),
                name,
            );
        }

        Ok(())
    }
}
