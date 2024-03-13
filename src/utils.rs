use super::error::RGitError;
use anyhow::Result;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_rgit_dir(root: &Path) -> Result<PathBuf> {
    let mut root = root.to_path_buf();
    loop {
        let rgit_dir = root.join(".rgit");
        if rgit_dir.is_dir() {
            return Ok(rgit_dir);
        }
        if !root.pop() {
            return Err(RGitError::new(
                "fatal: not a rgit repository (or any of the parent directories): .rgit"
                    .to_string(),
                128,
            ));
        }
    }
}

pub fn get_rgit_object_path(
    root: Option<&Path>,
    hash: &[u8; 20],
    check_exists: bool,
) -> Result<PathBuf> {
    let hash = hex::encode(hash);
    let object_path = get_rgit_dir(root)?
        .join("objects")
        .join(&hash[..2])
        .join(&hash[2..]);
    if check_exists && fs::metadata(&object_path).is_err() {
        return Err(RGitError::new(
            format!("fatal: Not a valid object name {}", hash),
            128,
        ));
    }
    Ok(object_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_rgit_dir() {
        let temp_dir = tempdir().unwrap();
        let rgit_dir = temp_dir.path().join(".rgit");
        fs::create_dir(&rgit_dir).unwrap();

        let result = get_rgit_dir(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), rgit_dir);

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_get_rgit_dir_error() {
        let temp_dir = tempdir().unwrap();

        let result = get_rgit_dir(Some(temp_dir.path()));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a rgit repository"));

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_get_rgit_object_path() {
        let temp_dir = tempdir().unwrap();
        let rgit_dir = temp_dir.path().join(".rgit");
        let objects_dir = rgit_dir.join("objects");
        fs::create_dir_all(&objects_dir).unwrap();

        let mut rng = rand::thread_rng();
        let hash: [u8; 20] = rng.gen();
        let hash_str = hex::encode(&hash);
        let object_path = objects_dir.join(&hash_str[..2]).join(&hash_str[2..]);

        let result = get_rgit_object_path(Some(temp_dir.path()), &hash, false);
        println!("{:?}", result);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), object_path);

        fs::create_dir_all(object_path.parent().unwrap()).unwrap();
        fs::File::create(&object_path).unwrap();

        let result = get_rgit_object_path(Some(temp_dir.path()), &hash, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), object_path);

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_get_rgit_object_path_error() {
        let temp_dir = tempdir().unwrap();
        let rgit_dir = temp_dir.path().join(".rgit");
        let objects_dir = rgit_dir.join("objects");
        fs::create_dir_all(&objects_dir).unwrap();

        let result = get_rgit_object_path(Some(temp_dir.path()), &[0u8; 20], true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not a valid object name"));

        temp_dir.close().unwrap();
    }
}
