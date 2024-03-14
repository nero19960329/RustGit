use super::super::utils::init_rgit_dir;
use anyhow::Result;
use std::env;
use std::fs;

pub fn rgit_init() -> Result<()> {
    let dir = env::current_dir()?;
    let rgit_dir_exist = fs::metadata(&dir.join(".rgit")).is_ok();
    let rgit_dir = init_rgit_dir(&dir)?;
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
