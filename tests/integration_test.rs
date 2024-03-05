use assert_cmd::Command;
use std::fs;
use std::str::from_utf8;
use tempfile;

fn rgit_command() -> Command {
    Command::cargo_bin("rgit").unwrap()
}

#[test]
fn test_rgit_init() {
    let dir = tempfile::tempdir().unwrap();
    rgit_command()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();

    assert!(dir.path().join(".rgit").exists());
}

#[test]
fn test_rgit_hash_object() {
    let dir = tempfile::tempdir().unwrap();

    // test non-existing file
    rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "test.txt"])
        .assert()
        .failure();

    // test existing file
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!").unwrap();
    rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "test.txt"])
        .assert()
        .success();

    // test write option without initializing the repository
    rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "-w", "test.txt"])
        .assert()
        .failure();

    // test write option
    rgit_command()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();
    rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "-w", "test.txt"])
        .assert()
        .success();

    // assert .rgit/objects directory
    let objects_dir = dir.path().join(".rgit").join("objects");
    assert!(objects_dir.exists());
}

#[test]
fn test_cat_file() {
    let dir = tempfile::tempdir().unwrap();

    // test under an un-initialized repository
    rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "invalid_hash"])
        .assert()
        .failure();

    rgit_command()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();

    let file_path = dir.path().join("test.txt");
    let content = "Hello, World!";
    fs::write(&file_path, content).unwrap();
    let output = rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "-w", "test.txt"])
        .output()
        .unwrap();
    let hash = from_utf8(&output.stdout).unwrap().trim().to_string();
    assert_eq!(hash.len(), 40);

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", &hash])
        .assert()
        .success();
    let cat_file_content = from_utf8(&result.get_output().stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(cat_file_content, content);

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-t", &hash])
        .assert()
        .success();
    let cat_file_type = from_utf8(&result.get_output().stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(cat_file_type, "blob");

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-s", &hash])
        .assert()
        .success();
    let cat_file_size = from_utf8(&result.get_output().stdout)
        .unwrap()
        .trim()
        .to_string();
    assert_eq!(cat_file_size, content.len().to_string());

    rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", "invalid_hash"])
        .assert()
        .failure();

    rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", &hash])
        .assert()
        .failure();
}

#[test]
fn test_write_tree() {
    let dir = tempfile::tempdir().unwrap();
    rgit_command()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();

    let file1_path = dir.path().join("file1.txt");
    let file2_path = dir.path().join("file2.txt");
    let subdir_path = dir.path().join("subdir");
    let file3_path = subdir_path.join("file3.txt");

    fs::write(&file1_path, "File 1 content").unwrap();
    fs::write(&file2_path, "File 2 content").unwrap();
    fs::create_dir(&subdir_path).unwrap();
    fs::write(&file3_path, "File 3 content").unwrap();

    let output = rgit_command()
        .current_dir(dir.path())
        .args(["write-tree"])
        .output()
        .unwrap();
    let tree_hash = from_utf8(&output.stdout).unwrap().trim().to_string();

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", &tree_hash])
        .assert()
        .success();
    let tree_content = from_utf8(&result.get_output().stdout)
        .unwrap()
        .trim()
        .to_string();

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

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", subdir_tree_hash])
        .assert()
        .success();
    let subdir_tree_content = from_utf8(&result.get_output().stdout)
        .unwrap()
        .trim()
        .to_string();

    assert!(subdir_tree_content.contains("100644 blob"));
    assert!(subdir_tree_content.contains("file3.txt"));
}
