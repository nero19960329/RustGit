use super::super::utils::init_rgit_dir;
use anyhow::Result;
use std::env;
use std::fs;
use std::io;
use std::path;

fn init(dir: &path::Path, writer: &mut dyn io::Write) -> Result<u8> {
    let rgit_dir_exist = fs::metadata(&dir.join(".rgit")).is_ok();
    let rgit_dir = init_rgit_dir(&dir)?;
    if !rgit_dir_exist {
        writeln!(
            writer,
            "Initialized empty RGit repository in {}",
            rgit_dir.display()
        )?;
    } else {
        writeln!(
            writer,
            "Reinitialized existing RGit repository in {}",
            rgit_dir.display()
        )?;
    }

    Ok(0)
}

pub fn rgit_init() -> Result<u8> {
    init(&env::current_dir()?, &mut io::stdout())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_rgit_init() {
        let dir = tempdir().unwrap();
        let mut buffer = Vec::new();
        let result = init(dir.path(), &mut buffer).unwrap();
        assert_eq!(result, 0);
        assert!(String::from_utf8(buffer)
            .unwrap()
            .contains("Initialized empty RGit repository"));

        let mut buffer = Vec::new();
        let result = init(dir.path(), &mut buffer).unwrap();
        assert_eq!(result, 0);
        assert!(String::from_utf8(buffer)
            .unwrap()
            .contains("Reinitialized existing RGit repository"));
    }
}
