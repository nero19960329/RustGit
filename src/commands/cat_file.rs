use super::super::error::RGitError;
use super::super::utils::get_rgit_dir;
use clap::{ArgGroup, Parser};
use std::fs::{self, File};
use std::io::{self, BufRead, Read};

/// Provide content for repository objects
#[derive(Parser, Debug)]
#[clap(group(ArgGroup::new("mode").required(true).args(&["t", "s", "p"])))]
pub struct CatFileArgs {
    /// Instead of the content, show the object type identified by <object>
    #[arg(name = "t", short)]
    pub t: bool,

    /// Instead of the content, show the object size identified by <object>
    #[arg(short)]
    pub s: bool,

    /// Pretty-print the contents of <object> based on its type
    #[arg(short)]
    pub p: bool,

    /// The name of the object to show
    pub object: String,
}

pub fn rgit_cat_file(args: &CatFileArgs) -> Result<(), Box<RGitError>> {
    let rgit_dir = get_rgit_dir()?;

    let object = &rgit_dir.join("objects").join(&args.object);
    if fs::metadata(&object).is_err() {
        return Err(Box::new(RGitError::new(
            format!("fatal: Not a valid object name {}", &args.object),
            128,
        )));
    }

    let object_file = File::open(&object).unwrap();
    let mut reader = io::BufReader::new(object_file);

    let mut header = Vec::new();
    reader.read_until(b'\x00', &mut header).unwrap();
    let binding = String::from_utf8(header).unwrap();
    let header = binding.trim_end_matches('\x00');
    let (object_type, object_size) = header.split_once(" ").unwrap();

    if args.t {
        println!("{}", object_type);
    } else if args.s {
        println!("{}", object_size);
    } else if args.p {
        assert!(object_type == "blob");
        let mut buffer = [0; 1024];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            let s = String::from_utf8_lossy(&buffer[..n]);
            print!("{}", s);
        }
    }

    Ok(())
}
