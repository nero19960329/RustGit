use crate::objects::Tree;
use crate::utils::get_rgit_dir;
use anyhow::Result;
use std::env;
use std::io;
use std::path;

fn write_tree(dir: &path::Path, writer: &mut dyn io::Write) -> Result<u8> {
    let rgit_dir = get_rgit_dir(dir)?;
    let tree = Tree::from_directory(dir)?;
    let tree_hash = tree.hash();
    tree.write_to_rgit_objects(rgit_dir.as_path())?;

    writeln!(writer, "{}", hex::encode(tree_hash))?;
    Ok(0)
}

pub fn rgit_write_tree() -> Result<u8> {
    write_tree(&env::current_dir()?, &mut io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::RGitObject;
    use crate::utils::init_rgit_dir;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_write_tree() {
        let dir = tempdir().unwrap();
        let rgit_dir = init_rgit_dir(dir.path()).unwrap();

        let file1_path = dir.path().join("file1.txt");
        let file2_path = dir.path().join("file2.txt");
        let subdir_path = dir.path().join("subdir");
        let file3_path = subdir_path.join("file3.txt");

        fs::write(&file1_path, "File 1 content").unwrap();
        fs::write(&file2_path, "File 2 content").unwrap();
        fs::create_dir(&subdir_path).unwrap();
        fs::write(&file3_path, "File 3 content").unwrap();

        let mut buffer = Vec::new();
        let result = write_tree(dir.path(), &mut buffer);
        assert!(result.is_ok());
        let tree_hash = String::from_utf8(buffer).unwrap();
        let tree_hash = hex::decode(tree_hash.trim()).unwrap();
        assert_eq!(tree_hash.len(), 20);
        let mut tree_hash_array = [0; 20];
        tree_hash_array.copy_from_slice(&tree_hash);

        let tree = Tree::from_rgit_objects(rgit_dir.as_path(), &tree_hash_array).unwrap();
        let mut buffer = Vec::new();
        tree.print(&mut buffer).unwrap();
        let tree_content = String::from_utf8(buffer).unwrap();

        assert!(tree_content.contains("100644 blob"));
        assert!(tree_content.contains("file1.txt"));
        assert!(tree_content.contains("file2.txt"));
        assert!(tree_content.contains("040000 tree"));
        assert!(tree_content.contains("subdir"));

        let subdir_tree_hash = tree_content
            .lines()
            .find(|line| line.contains("subdir"))
            .unwrap()
            .split_whitespace()
            .nth(2)
            .unwrap();
        let subdir_tree_hash = hex::decode(subdir_tree_hash).unwrap();
        assert_eq!(subdir_tree_hash.len(), 20);
        let mut subdir_tree_hash_array = [0; 20];
        subdir_tree_hash_array.copy_from_slice(&subdir_tree_hash);

        let subdir_tree =
            Tree::from_rgit_objects(rgit_dir.as_path(), &subdir_tree_hash_array).unwrap();
        let mut buffer = Vec::new();
        subdir_tree.print(&mut buffer).unwrap();
        let subdir_tree_content = String::from_utf8(buffer).unwrap();

        assert!(subdir_tree_content.contains("100644 blob"));
        assert!(subdir_tree_content.contains("file3.txt"));
    }
}
