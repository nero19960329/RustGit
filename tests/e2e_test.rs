use assert_cmd::Command;
use std::fs;
use std::str::from_utf8;
use tempfile;

fn rgit_command() -> Command {
    Command::cargo_bin("rgit").unwrap()
}

#[test]
fn test_rgit_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    rgit_command()
        .current_dir(dir.path())
        .args(["init"])
        .assert()
        .success();

    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!").unwrap();
    let result = rgit_command()
        .current_dir(dir.path())
        .args(["hash-object", "-w", "test.txt"])
        .assert()
        .success();
    let hash = from_utf8(&result.get_output().stdout).unwrap().trim();

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", hash])
        .assert()
        .success();
    let content = from_utf8(&result.get_output().stdout).unwrap().trim();
    assert_eq!(content, "Hello, World!");

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["write-tree"])
        .assert()
        .success();
    let tree_hash = from_utf8(&result.get_output().stdout).unwrap().trim();

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", tree_hash])
        .assert()
        .success();
    let tree_content = from_utf8(&result.get_output().stdout).unwrap().trim();

    assert!(tree_content.contains("100644 blob"));
    assert!(tree_content.contains("test.txt"));

    let subdir_path = dir.path().join("subdir");
    fs::create_dir(&subdir_path).unwrap();
    let file_path = subdir_path.join("subfile.txt");
    fs::write(&file_path, "Subdir file content").unwrap();

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["write-tree"])
        .assert()
        .success();
    let tree_hash = from_utf8(&result.get_output().stdout).unwrap().trim();

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["cat-file", "-p", tree_hash])
        .assert()
        .success();
    let tree_content = from_utf8(&result.get_output().stdout).unwrap().trim();

    assert!(tree_content.contains("100644 blob"));
    assert!(tree_content.contains("test.txt"));
    assert!(tree_content.contains("040000 tree"));
    assert!(tree_content.contains("subdir"));

    rgit_command()
        .current_dir(dir.path())
        .args(["read-tree", tree_hash])
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(dir.path().join("test.txt")).unwrap(),
        "Hello, World!"
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("subdir/subfile.txt")).unwrap(),
        "Subdir file content"
    );

    let result = rgit_command()
        .current_dir(dir.path())
        .args(["commit", "-m", "Initial commit"])
        .assert()
        .success();

    let commit_content = from_utf8(&result.get_output().stdout).unwrap().trim();
    assert!(commit_content.starts_with("[commit"));
    assert!(commit_content.contains("Initial commit"));
}
