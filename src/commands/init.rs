use super::super::utils::get_rgit_dir;
use anyhow::Result;
use std::env;
use std::fs;
use std::path;

fn init_rgit_dir(root: &path::Path) -> Result<()> {
    println!("root: {:?}", root);
    let rgit_dir = get_rgit_dir(Some(root));
    println!("(1) rgit_dir: {:?}", rgit_dir);
    if rgit_dir.is_ok() {
        println!(
            "Reinitialized existing RGit repository in {}",
            rgit_dir.unwrap().display()
        );
        return Ok(());
    }

    let rgit_dir = root.join(".rgit");
    println!("(2) rgit_dir: {:?}", rgit_dir);
    fs::create_dir(&rgit_dir)?;
    println!("create_dir ok!");
    println!(
        "Initialized empty RGit repository in {}",
        rgit_dir.display()
    );
    Ok(())
}

pub fn rgit_init() -> Result<()> {
    init_rgit_dir(&env::current_dir()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_init_rgit_dir() {
        let temp_dir = tempdir().unwrap();
        let rgit_dir = temp_dir.path().join(".rgit");
        let result = init_rgit_dir(temp_dir.path());
        assert!(result.is_ok());
        assert!(fs::metadata(&rgit_dir).is_ok());
        temp_dir.close().unwrap();
    }

    #[test]
    fn test_init_rgit_dir_already_exists() {
        let temp_dir = tempdir().unwrap();
        let rgit_dir = temp_dir.path().join(".rgit");
        fs::create_dir(&rgit_dir).unwrap();
        println!("{:?}", rgit_dir);
        let result = init_rgit_dir(temp_dir.path());
        println!("{:?}", result);
        assert!(result.is_ok());
        temp_dir.close().unwrap();
    }
}
