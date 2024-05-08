use crate::hash::hash_object;
use crate::ignore::is_ignored;
use crate::objects::blob::Blob;
use crate::objects::{RGitObject, RGitObjectHeader, RGitObjectType};
use crate::utils::get_rgit_object_path;
use anyhow::Result;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str;

#[derive(Debug, PartialEq)]
pub enum EntryType {
    Regular,
    Executable,
    Tree,
    Symlink,
}

impl EntryType {
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "100644" => Ok(EntryType::Regular),
            "100755" => Ok(EntryType::Executable),
            "040000" => Ok(EntryType::Tree),
            "120000" => Ok(EntryType::Symlink),
            _ => Err(anyhow::anyhow!("Invalid entry type: {}", s)),
        }
    }

    fn to_string(&self) -> String {
        match self {
            EntryType::Regular => "100644".to_string(),
            EntryType::Executable => "100755".to_string(),
            EntryType::Tree => "040000".to_string(),
            EntryType::Symlink => "120000".to_string(),
        }
    }
}

#[derive(Debug)]
enum EntryObject {
    Blob(Blob),
    Tree(Tree),
}

#[derive(Debug)]
pub struct Tree {
    entries: BTreeMap<String, Entry>,
    hash: [u8; 20],
}

#[derive(Debug)]
struct Entry {
    entry_type: EntryType,
    name: String,
    object: EntryObject,
}

impl Tree {
    fn new(entries: BTreeMap<String, Entry>) -> Result<Self> {
        let mut content = Vec::new();

        for (name, entry) in &entries {
            match &entry.object {
                EntryObject::Blob(blob) => {
                    content
                        .extend(format!("{} {}\0", entry.entry_type.to_string(), name).as_bytes());
                    content.extend(blob.hash());
                }
                EntryObject::Tree(tree) => {
                    content
                        .extend(format!("{} {}\0", entry.entry_type.to_string(), name).as_bytes());
                    content.extend(tree.hash());
                }
            }
        }

        let hash = hash_object(io::Cursor::new(content))?;

        Ok(Self { entries, hash })
    }

    pub fn from_directory(path: &Path) -> Result<Self> {
        let mut entries = BTreeMap::new();

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let entry_path = entry.path();

            // ignore .rgit directory
            if entry_path.ends_with(".rgit") {
                continue;
            }
            if is_ignored(&entry_path)?.is_ignored {
                continue;
            }

            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("Invalid file name: {:?}", entry.file_name()))?;

            let entry_type = if file_type.is_dir() {
                EntryType::Tree
            } else if file_type.is_symlink() {
                EntryType::Symlink
            } else if file_type.is_file() {
                let metadata = entry.metadata()?;
                let mode = metadata.permissions().mode();
                if mode & 0o111 != 0 {
                    EntryType::Executable
                } else {
                    EntryType::Regular
                }
            } else {
                return Err(anyhow::anyhow!("Unsupported entry type: {:?}", entry_path));
            };

            let object = match entry_type {
                EntryType::Regular | EntryType::Executable => {
                    EntryObject::Blob(Blob::from_file(&entry_path)?)
                }
                EntryType::Tree => EntryObject::Tree(Tree::from_directory(&entry_path)?),
                EntryType::Symlink => {
                    return Err(anyhow::anyhow!("Symlink is not supported yet"));
                }
            };

            let entry_name = name.clone();
            entries.insert(
                name,
                Entry {
                    entry_type,
                    name: entry_name,
                    object,
                },
            );
        }

        Tree::new(entries)
    }

    pub fn write_to_directory(&self, path: &Path) -> Result<()> {
        for (name, entry) in &self.entries {
            let entry_path = path.join(name);
            match &entry.object {
                EntryObject::Blob(blob) => {
                    blob.write_to_file(&entry_path)?;
                }
                EntryObject::Tree(tree) => {
                    fs::create_dir(&entry_path)?;
                    tree.write_to_directory(&entry_path)?;
                }
            }
        }

        Ok(())
    }

    pub fn from_rgit_objects(rgit_dir: &Path, hash: &[u8; 20]) -> Result<Self> {
        let object_path = get_rgit_object_path(rgit_dir, hash, true)?;
        let mut reader = fs::File::open(&object_path)?;

        let header = RGitObjectHeader::deserialize(&mut reader)?;
        let mut content = vec![0; header.content_size];
        reader.read_exact(&mut content)?;

        let mut entries = BTreeMap::new();
        let mut cursor = 0;

        while cursor < content.len() {
            let space_pos = content[cursor..]
                .iter()
                .position(|&x| x == b' ')
                .ok_or(anyhow::anyhow!("Invalid tree entry"))?;
            let mode = str::from_utf8(&content[cursor..cursor + space_pos])?.to_string();
            cursor += space_pos + 1;

            let null_pos = content[cursor..]
                .iter()
                .position(|&x| x == 0)
                .ok_or(anyhow::anyhow!("Invalid tree entry"))?;
            let name = str::from_utf8(&content[cursor..cursor + null_pos])?.to_string();
            cursor += null_pos + 1;

            let hash = <[u8; 20]>::try_from(&content[cursor..cursor + 20])
                .map_err(|_| anyhow::anyhow!("Invalid tree entry"))?;
            cursor += 20;

            let entry_type = EntryType::from_str(&mode)?;

            let entry_object: EntryObject = match entry_type {
                EntryType::Regular | EntryType::Executable => {
                    EntryObject::Blob(Blob::from_rgit_objects(rgit_dir, &hash)?)
                }
                EntryType::Tree => EntryObject::Tree(Tree::from_rgit_objects(rgit_dir, &hash)?),
                EntryType::Symlink => {
                    return Err(anyhow::anyhow!("Symlink is not supported yet"));
                }
            };

            entries.insert(
                name.clone(),
                Entry {
                    entry_type,
                    name,
                    object: entry_object,
                },
            );
        }

        Tree::new(entries)
    }

    pub fn write_to_rgit_objects(&self, rgit_dir: &Path) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, &self.hash, false)?;
        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut file = fs::File::create(&object_path)?;
        self.serialize(&mut file)?;

        for (_, entry) in &self.entries {
            match &entry.object {
                EntryObject::Blob(blob) => {
                    blob.write_to_rgit_objects(rgit_dir)?;
                }
                EntryObject::Tree(tree) => {
                    tree.write_to_rgit_objects(rgit_dir)?;
                }
            }
        }

        Ok(())
    }
}

impl RGitObject for Tree {
    fn object_type(&self) -> RGitObjectType {
        RGitObjectType::Tree
    }

    fn size(&self) -> usize {
        let mut size = 0;
        for (_, entry) in &self.entries {
            size += entry.entry_type.to_string().len() + 1 + entry.name.len() + 1 + 20;
        }
        size
    }

    fn hash(&self) -> &[u8; 20] {
        &self.hash
    }

    fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
        let mut content = Vec::new();
        for (name, entry) in &self.entries {
            match &entry.object {
                EntryObject::Blob(blob) => {
                    content.extend(
                        format!("{} {}\0", EntryType::Regular.to_string(), name).as_bytes(),
                    );
                    content.extend(blob.hash());
                }
                EntryObject::Tree(tree) => {
                    content
                        .extend(format!("{} {}\0", EntryType::Tree.to_string(), name).as_bytes());
                    content.extend(tree.hash());
                }
            }
        }

        let header = RGitObjectHeader::new(self.object_type(), content.len());
        header.serialize(writer)?;
        writer.write_all(&content)?;
        Ok(())
    }

    fn print(&self, writer: &mut dyn Write) -> Result<()> {
        for (name, entry) in &self.entries {
            let rgit_object_type = match &entry.object {
                EntryObject::Blob(blob) => blob.object_type(),
                EntryObject::Tree(tree) => tree.object_type(),
            };
            let rgit_object_hash = match &entry.object {
                EntryObject::Blob(blob) => blob.hash(),
                EntryObject::Tree(tree) => tree.hash(),
            };

            writer.write_all(
                format!(
                    "{} {} {}\t{}\n",
                    entry.entry_type.to_string(),
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
    fn test_tree_from_directory() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_directory(path).unwrap();
        assert_eq!(tree.entries.len(), 2);
    }

    #[test]
    fn test_tree_from_rgit_objects() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let rgit_dir = init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_directory(path).unwrap();
        tree.write_to_rgit_objects(rgit_dir.as_path()).unwrap();

        let tree = Tree::from_rgit_objects(rgit_dir.as_path(), tree.hash()).unwrap();
        assert_eq!(tree.entries.len(), 2);
    }

    #[test]
    fn test_tree_print() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        init_rgit_dir(path).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_directory(path).unwrap();
        let mut buffer = Vec::new();
        tree.print(&mut buffer).unwrap();
    }
}
