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
}
