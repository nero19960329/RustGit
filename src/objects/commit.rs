use super::rgit_object::{RGitObject, RGitObjectHeader, RGitObjectType};
use super::super::hash::hash;
use super::super::utils::get_rgit_object_path;
use anyhow::Result;
use std::fs;
use std::fmt;
use std::io::{self, BufRead, Read, Write};
use std::path;

#[derive(Debug)]
pub struct Commit {
    hash: [u8; 20],

    tree_hash: [u8; 20],
    parent_hash: [u8; 20],
    message: String,
}

impl Commit {
    pub fn new(tree_hash: [u8; 20], parent_hash: [u8; 20], message: String) -> Result<Commit> {
        let mut commit = Commit {
            hash: [0; 20],
            tree_hash,
            parent_hash,
            message,
        };
        let commit_string = commit.to_string();
        let reader = commit_string.as_bytes();
        let hash = hash(vec![reader].into_iter())?;
        commit.hash = hash;
        Ok(commit)
    }

    pub fn from_hash(rgit_dir: &path::Path, hash: [u8; 20]) -> Result<Commit> {
        let object_path = get_rgit_object_path(rgit_dir, &hash, true)?;
        let mut object_file = fs::File::open(&object_path)?;
        let mut reader = io::BufReader::new(&mut object_file);
        let header = RGitObjectHeader::deserialize(&mut reader)?;
        if header.object_type != RGitObjectType::Commit {
            return Err(anyhow::anyhow!("Object is not a commit"));
        }

        // first line: `tree {tree_hash}`
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let tree_hash = hex::decode(&line[5..45])?;

        // second line: `parent {parent_hash}`
        line.clear();
        reader.read_line(&mut line)?;
        let parent_hash = hex::decode(&line[7..47])?;

        // third line: empty
        line.clear();
        reader.read_line(&mut line)?;

        // rest of the lines: message
        let mut message = String::new();
        reader.read_to_string(&mut message)?;

        Ok(Commit {
            hash,
            tree_hash: tree_hash.try_into().unwrap(),
            parent_hash: parent_hash.try_into().unwrap(),
            message,
        })
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // tree {tree_hash}
        // parent {parent_hash}
        // 
        // {message}
        write!(
            f,
            "tree {}\nparent {}\n\n{}",
            hex::encode(self.tree_hash),
            hex::encode(self.parent_hash),
            self.message,
        )
    }
}

impl RGitObject for Commit {
    fn header(&self) -> Result<RGitObjectHeader> {
        let size = self.to_string().len();

        Ok(RGitObjectHeader::new(
            RGitObjectType::Commit,
            size,
        ))
    }

    fn hash(&self) -> Result<&[u8; 20]> {
        Ok(&self.hash)
    }

    fn write(&self, _rgit_dir: &path::Path, _path: &path::Path) -> Result<()> {
        unimplemented!()
    }

    fn write_object(&self, rgit_dir: &path::Path) -> Result<()> {
        let object_path = get_rgit_object_path(rgit_dir, &self.hash, false)?;

        fs::create_dir_all(object_path.parent().unwrap())?;
        let mut object_file = fs::File::create(&object_path)?;
        self.serialize(&mut object_file)?;

        let head_path = rgit_dir.join("HEAD");
        let mut head_file = fs::File::create(&head_path)?;
        write!(head_file, "{}", hex::encode(self.hash))?;

        Ok(())
    }

    fn serialize(&self, writer: &mut dyn std::io::Write) -> Result<()> {
        write!(writer, "{}", self)?;
        Ok(())
    }

    fn serialize_object(&self, _rgit_dir: &path::Path, writer: &mut dyn std::io::Write) -> Result<()> {
        write!(writer, "{}", self)?;
        Ok(())
    }
}
