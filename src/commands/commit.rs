use crate::hash::hash_array_from_str;
use crate::objects::Commit;
use crate::objects::Tree;
use crate::utils::get_rgit_dir;
use anyhow::Result;
use clap::Parser;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

/// Record changes to the repository
#[derive(Parser, Debug)]
pub struct CommitArgs {
    /// Use the given <msg> as the commit message.
    #[arg(name = "message", short, long)]
    pub message: String,
}

fn get_head(rgit_dir: &Path) -> Result<Option<[u8; 20]>> {
    let head_path = rgit_dir.join("HEAD");
    if !head_path.exists() {
        return Ok(None);
    }

    // XXX: .rgit/HEAD should be a symbolic link to refs/heads/master
    //      but we are going to use a file with the hash of the commit
    let hash = fs::read_to_string(head_path)?;
    let hash = hash.trim();
    let hash = hash_array_from_str(hash)?;
    Ok(Some(hash))
}

fn set_head(rgit_dir: &Path, hash: &[u8; 20]) -> Result<()> {
    let head_path = rgit_dir.join("HEAD");
    // XXX: .rgit/HEAD should be a symbolic link to refs/heads/master
    //      but we are going to use a file with the hash of the commit
    fs::write(head_path, hex::encode(hash))?;
    Ok(())
}

fn commit(dir: &Path, message: String, writer: &mut dyn io::Write) -> Result<u8> {
    let rgit_dir = get_rgit_dir(dir)?;

    let tree = Tree::from_directory(dir)?;
    let mut tree_hash: [u8; 20] = [0; 20];
    tree_hash.copy_from_slice(tree.hash());
    tree.write_to_rgit_objects(rgit_dir.as_path())?;

    let parent = get_head(&rgit_dir);
    let parents = match parent {
        Ok(Some(hash)) => vec![hash],
        _ => vec![],
    };

    let commit = Commit::new(tree_hash, parents, message.clone())?;
    commit.write_to_rgit_objects(&rgit_dir)?;

    set_head(&rgit_dir, &commit.hash()?)?;

    let commit_hash_prefix = hex::encode(commit.hash()?)
        .chars()
        .take(7)
        .collect::<String>();
    writeln!(writer, "[commit {}] {}", commit_hash_prefix, message,)?;
    // XXX: print the diff

    Ok(0)
}

pub fn rgit_commit(args: &CommitArgs) -> Result<u8> {
    commit(
        &env::current_dir()?,
        args.message.clone(),
        &mut io::stdout(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::init_rgit_dir;
    use rand::Rng;
    use tempfile::tempdir;

    #[test]
    fn test_get_head() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();
        let head = get_head(&rgit_dir).unwrap();
        assert_eq!(head, None);

        let hash = rand::thread_rng().gen::<[u8; 20]>();
        set_head(&rgit_dir, &hash).unwrap();
        let head = get_head(&rgit_dir).unwrap();
        assert_eq!(head, Some(hash));
    }

    #[test]
    fn test_commit() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();
        let message = "Initial commit".to_string();

        let mut buffer = Vec::new();
        let result = commit(dir.path(), message.clone(), &mut buffer).unwrap();
        assert_eq!(result, 0);

        let commit_content = String::from_utf8(buffer).unwrap();
        assert!(commit_content.starts_with("[commit "));
        assert!(commit_content.contains(&message));

        let commit =
            Commit::from_rgit_objects(&rgit_dir, &get_head(&rgit_dir).unwrap().unwrap()).unwrap();
        assert_eq!(commit.commit_message, message);
    }
}
