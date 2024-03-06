use anyhow::Result;
use std::env;
use std::fs;

pub fn rgit_init() -> Result<()> {
    let rgit_dir = env::current_dir()?.join(".rgit");
    let rgit_dir_exist = fs::metadata(&rgit_dir).is_ok();
    if !rgit_dir_exist {
        fs::create_dir(".rgit")?;
    }
    if fs::metadata(".rgit/objects").is_err() {
        fs::create_dir(".rgit/objects")?;
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
