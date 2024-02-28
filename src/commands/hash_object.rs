use clap::Parser;
use sha1::{Digest, Sha1};
use std::{env, fs, process};

/// Compute object ID
#[derive(Parser, Debug)]
pub struct HashObjectArgs {
    /// Actually write the object into the object database
    #[arg(name = "write", short)]
    pub write: bool,

    pub file: String,
}

pub fn rgit_hash_object(args: &HashObjectArgs) {
    let rgit_dir = env::current_dir().unwrap().join(".rgit");
    let file = env::current_dir().unwrap().join(&args.file);
    if fs::metadata(&file).is_err() {
        eprintln!(
            "fatal: could not open '{}' for reading: No such file or directory",
            &args.file
        );
        process::exit(128);
    }

    let content = fs::read_to_string(&file).unwrap();
    let size = content.len();
    let data = [
        &b"blob "[..],
        &b" "[..],
        size.to_string().as_bytes(),
        &b"\x00"[..],
        &content.as_bytes(),
    ]
    .concat();
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash_result = hasher.finalize();
    let hash = format!("{:x}", hash_result);
    let object = rgit_dir.join("objects").join(&hash);
    if args.write {
        if fs::metadata(rgit_dir).is_ok() {
            fs::write(&object, &data).unwrap();
        } else {
            eprintln!("fatal: not an rgit repository (or any of the parent directories): .rgit");
            process::exit(128);
        }
    }
    println!("{}", hash);
}
