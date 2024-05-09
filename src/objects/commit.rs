use crate::hash::hash_object;
use crate::objects::{RGitObject, RGitObjectHeader, RGitObjectType};
use crate::utils::get_rgit_object_path;
use anyhow::Result;
use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug)]
pub struct Commit {
    tree: [u8; 20],
    parents: Vec<[u8; 20]>,
    time: DateTime<FixedOffset>,
    pub commit_message: String,
}

fn parse_timezone_offset(offset_str: &str) -> Result<FixedOffset> {
    let offset_sign = offset_str
        .chars()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid timezone offset"))?;
    let offset_hours = offset_str[1..3]
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Invalid timezone offset hours"))?;
    let offset_minutes = offset_str[3..5]
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Invalid timezone offset minutes"))?;

    let offset_secs = if offset_sign == '+' {
        offset_hours * 3600 + offset_minutes * 60
    } else if offset_sign == '-' {
        -(offset_hours * 3600 + offset_minutes * 60)
    } else {
        return Err(anyhow::anyhow!("Invalid timezone offset sign"));
    };

    FixedOffset::east_opt(offset_secs).ok_or_else(|| anyhow::anyhow!("Invalid timezone offset"))
}

fn serialize_timezone_offset(offset: &FixedOffset) -> String {
    let offset_str = offset.to_string();
    let offset_sign = offset_str.chars().next().unwrap();
    let offset_hours = offset_str[1..3].parse::<i32>().unwrap();
    let offset_minutes = offset_str[4..6].parse::<i32>().unwrap();

    format!("{}{:02}{:02}", offset_sign, offset_hours, offset_minutes)
}

impl Commit {
    pub fn new(tree: [u8; 20], parents: Vec<[u8; 20]>, commit_message: String) -> Result<Self> {
        let now = Local::now();
        let offset = now.offset().fix();
        let time = offset.from_local_datetime(&now.naive_local()).single();
        if time.is_none() {
            return Err(anyhow::anyhow!("Invalid timestamp"));
        }
        let time = time.unwrap();

        Ok(Self {
            tree,
            parents,
            time,
            commit_message,
        })
    }

    pub fn from_rgit_objects(rgit_dir: &Path, hash: &[u8; 20]) -> Result<Self> {
        let object_path = get_rgit_object_path(rgit_dir, hash, true)?;
        let mut reader = fs::File::open(object_path)?;

        let header = RGitObjectHeader::deserialize(&mut reader)?;
        if header.object_type != RGitObjectType::Commit {
            return Err(anyhow::anyhow!(
                "Invalid object type: {:?}",
                header.object_type
            ));
        }

        let mut content = String::new();
        reader.read_to_string(&mut content)?;

        let mut lines = content.lines();

        let tree_line = lines.next().unwrap();
        let tree = hex::decode(&tree_line[5..])?.as_slice().try_into()?;

        let mut parents = Vec::new();
        let mut line = lines.next();
        while let Some(parent_line) = line {
            if !parent_line.starts_with("parent ") {
                break;
            }

            let parent: [u8; 20] = hex::decode(&parent_line[7..])?.as_slice().try_into()?;
            parents.push(parent);
            line = lines.next();
        }

        // time_line should be `line`, cannot be `lines.next()`
        let time_line = line.unwrap();
        let time_parts: Vec<&str> = time_line.split_whitespace().collect();
        if time_parts.len() != 3 || time_parts[0] != "time" {
            return Err(anyhow::anyhow!("Invalid time line: {:?}", time_line));
        }
        let timestamp = time_parts[1].parse::<i64>()?;
        let offset = parse_timezone_offset(time_parts[2])?;
        let time = offset
            .timestamp_opt(timestamp, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

        lines.next();
        let commit_message = lines.collect::<Vec<&str>>().join("\n");

        Ok(Self {
            tree,
            parents,
            time,
            commit_message,
        })
    }

    pub fn hash(&self) -> Result<[u8; 20]> {
        hash_object(self.content().as_bytes())
    }

    pub fn write_to_rgit_objects(&self, rgit_dir: &Path) -> Result<[u8; 20]> {
        let hash = self.hash()?;
        let object_path = get_rgit_object_path(rgit_dir, &hash, false)?;
        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut writer = fs::File::create(object_path)?;

        self.serialize(&mut writer)?;

        Ok(hash)
    }

    fn content(&self) -> String {
        let mut content = String::new();
        content.push_str(format!("tree {}\n", hex::encode(self.tree)).as_str());
        for parent in self.parents.iter() {
            content.push_str(format!("parent {}\n", hex::encode(parent)).as_str());
        }
        let offset = self.time.offset();
        let offset_string = serialize_timezone_offset(&offset);
        content.push_str(format!("time {} {}\n", self.time.timestamp(), offset_string).as_str());
        content.push_str("\n");
        content.push_str(self.commit_message.as_str());

        content
    }
}

impl RGitObject for Commit {
    fn object_type(&self) -> super::RGitObjectType {
        RGitObjectType::Commit
    }

    fn size(&self) -> usize {
        self.content().len()
    }

    fn serialize(&self, writer: &mut dyn Write) -> Result<()> {
        let header = RGitObjectHeader::new(self.object_type(), self.size());
        header.serialize(writer)?;
        writer.write_all(self.content().as_bytes())?;
        Ok(())
    }

    fn print(&self, writer: &mut dyn Write) -> Result<()> {
        let content = self.content();
        writer.write_all(content.as_bytes())?;
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
    fn test_parse_timezone_offset() {
        let offset = parse_timezone_offset("+0900").unwrap();
        assert_eq!(offset.to_string(), "+09:00");

        let offset = parse_timezone_offset("-0900").unwrap();
        assert_eq!(offset.to_string(), "-09:00");
    }

    #[test]
    fn test_serialize_timezone_offset() {
        let offset = FixedOffset::east_opt(9 * 3600).unwrap();
        assert_eq!(serialize_timezone_offset(&offset), "+0900");

        let offset = FixedOffset::west_opt(9 * 3600).unwrap();
        assert_eq!(serialize_timezone_offset(&offset), "-0900");
    }

    #[test]
    fn test_from_rgit_objects() {
        let dir = tempdir().unwrap();
        let path = dir.path();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        let file_path = path.join("file");
        fs::write(&file_path, "file content").unwrap();

        let subdir_path = path.join("dir");
        fs::create_dir(&subdir_path).unwrap();
        let subfile_path = subdir_path.join("subfile");
        fs::write(&subfile_path, "subfile content").unwrap();

        let tree = Tree::from_directory(path).unwrap();
        tree.write_to_rgit_objects(rgit_dir.as_path()).unwrap();

        let mut tree_hash = [0; 20];
        tree_hash.copy_from_slice(tree.hash());

        let commit = Commit::new(tree_hash, Vec::new(), "Initial commit".to_string()).unwrap();
        let commit_hash = commit.write_to_rgit_objects(rgit_dir.as_path()).unwrap();

        let commit = Commit::from_rgit_objects(rgit_dir.as_path(), &commit_hash).unwrap();
        assert_eq!(commit.tree, tree_hash);
        assert_eq!(commit.parents.len(), 0);
        assert_eq!(commit.commit_message, "Initial commit");
    }
}
