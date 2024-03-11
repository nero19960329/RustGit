use super::super::utils::get_rgit_dir;
use anyhow::Result;
use std::env;
use std::fs;
use std::path;

fn init_rgit_dir(root: &path::Path) -> Result<()> {
    let rgit_dir = get_rgit_dir(Some(root));
    if rgit_dir.is_ok() {
        println!(
            "Reinitialized existing RGit repository in {}",
            rgit_dir.unwrap().display()
        );
        return Ok(());
    }

    let rgit_dir = root.join(".rgit");
    fs::create_dir(&rgit_dir)?;
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
        let result = init_rgit_dir(temp_dir.path());
        assert!(result.is_ok());
        temp_dir.close().unwrap();
    }
}
