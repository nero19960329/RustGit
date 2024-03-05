use super::super::error::RGitError;
use super::super::hash::hash_object;
use super::super::ignore::{load_ignore_rules, RGitIgnore};
use std::collections::BTreeMap;
use std::env;
use std::fs::{read, read_dir};
use std::path::Path;

pub fn rgit_write_tree() -> Result<(), Box<RGitError>> {
    let current_dir = env::current_dir().unwrap();
    let ignore_files = RGitIgnore::load_ignore_files(&current_dir);
    let rgitignore = load_ignore_rules(&ignore_files);

    let tree_entries = build_tree_entries(&current_dir, &rgitignore)?;
    let tree_content = generate_tree_content(&tree_entries);
    let tree_hash = hash_object(&tree_content, "tree", true)?;

    println!("{}", tree_hash);
    Ok(())
}

fn build_tree_entries(
    dir: &Path,
    rgitignore: &RGitIgnore,
) -> Result<BTreeMap<String, String>, Box<RGitError>> {
    let mut entries = BTreeMap::new();

    for entry in read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if !rgitignore.is_ignored(&path, dir) {
            let file_type = entry.file_type().unwrap();
            let mode = if file_type.is_dir() {
                "040000"
            } else {
                "100644"
            };
            let object_type = if file_type.is_dir() { "tree" } else { "blob" };
            let hash = if file_type.is_dir() {
                let sub_entries = build_tree_entries(&path, rgitignore)?;
                let sub_tree_content = generate_tree_content(&sub_entries);
                hash_object(&sub_tree_content, "tree", true)?
            } else {
                hash_object(&read(&path).unwrap(), "blob", true)?
            };

            entries.insert(
                entry.file_name().to_string_lossy().to_string(),
                format!("{} {} {}", mode, object_type, hash),
            );
        }
    }

    Ok(entries)
}

fn generate_tree_content(entries: &BTreeMap<String, String>) -> Vec<u8> {
    let mut content = Vec::new();
    for (name, entry) in entries {
        let entry_content = format!("{} {}\x00", entry, name);
        content.extend_from_slice(entry_content.as_bytes());
    }
    content
}
