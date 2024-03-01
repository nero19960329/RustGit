use super::super::error::RGitError;
use std::env;
use std::fs;

pub fn rgit_init() -> Result<(), Box<RGitError>> {
    let rgit_dir = env::current_dir().unwrap().join(".rgit");
    let rgit_dir_exist = fs::metadata(&rgit_dir).is_ok();
    if !rgit_dir_exist {
        fs::create_dir(".rgit").unwrap();
    }
    if fs::metadata(".rgit/objects").is_err() {
        fs::create_dir(".rgit/objects").unwrap();
    }

    if !rgit_dir_exist {
        println!(
            "Initialized empty RGit repository in {}",
            rgit_dir.display()
        );
    } else {
        println!(
            "Reinitialized existing RGit repository in {}",
            rgit_dir.display()
        );
    }

    Ok(())
}
