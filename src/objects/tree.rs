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
use std::path::Path;

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

            // ignore .rgit directory
            if entry_path.ends_with(".rgit") {
                continue;
            }
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

        Ok(Self {
            hash: hash(vec![content.as_slice()].into_iter())?,
            entries: entries,
            content: content,
        })
    }

    pub fn from_hash(rgit_dir: &Path, hash: [u8; 20]) -> Result<Self> {
        let object_path = get_rgit_object_path(rgit_dir, &hash, true)?;
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
                EntryType::Tree => {
                    Box::new(Tree::from_hash(rgit_dir, hash)?) as Box<dyn RGitObject>
                }
                EntryType::Regular | EntryType::Executable => {
                    Box::new(Blob::from_hash(rgit_dir, hash)?) as Box<dyn RGitObject>
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

    fn write(&self, rgit_dir: &Path, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        for (name, tree_entry) in &self.entries {
            tree_entry.rgit_object.write(rgit_dir, &path.join(name))?;
        }

        Ok(())
    }

    fn write_object(&self, rgit_dir: &Path) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, self.hash()?, false)?;

        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut object_file = fs::File::create(&object_path)?;
        object_file.write_all(&self.header()?.serialize())?;
        object_file.write_all(&self.content)?;

        for (_, tree_entry) in &self.entries {
            tree_entry.rgit_object.write_object(rgit_dir)?;
        }

        Ok(())
    }

    fn serialize_object(&self, _rgit_dir: &Path, writer: &mut dyn Write) -> Result<()> {
        for (name, tree_entry) in &self.entries {
            let rgit_object_type = match tree_entry.rgit_object.header()? {
                RGitObjectHeader { object_type, .. } => object_type,
            };
            let rgit_object_hash = tree_entry.rgit_object.hash()?;

            writer.write_all(
                format!(
                    "{} {} {}\t{}\n",
                    tree_entry.entry_type.as_str(),
                    rgit_object_type,
                    hex::encode(rgit_object_hash),
                    name
                )
                .as_bytes(),
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_tree_from_path() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_path(path).unwrap();
        assert_eq!(tree.entries.len(), 2);
    }

    #[test]
    fn test_tree_from_hash() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let rgit_dir = init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_path(path).unwrap();
        tree.write_object(rgit_dir.as_path()).unwrap();

        let tree = Tree::from_hash(rgit_dir.as_path(), *tree.hash().unwrap()).unwrap();
        assert_eq!(tree.entries.len(), 2);
    }

    #[test]
    fn test_tree_serialize_object() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let rgit_dir = init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_path(path).unwrap();
        let mut buffer = Vec::new();
        tree.serialize_object(rgit_dir.as_path(), &mut buffer)
            .unwrap();
    }
}
