use super::super::error::RGitError;
use super::super::utils::get_rgit_dir;
use clap::{ArgGroup, Parser};
use std::fs::{self, File};
use std::io::{self, copy, stdout, BufRead, Read};

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
    let header = String::from_utf8(header).unwrap();

    let (object_type, object_size) = header.trim_end_matches('\x00').split_once(' ').unwrap();

    if args.t {
        println!("{}", object_type);
    } else if args.s {
        println!("{}", object_size);
    } else if args.p {
        match object_type {
            "blob" => print_blob_content(&mut reader),
            "tree" => print_tree_content(&mut reader)?,
            _ => {
                return Err(Box::new(RGitError::new(
                    format!(
                        "fatal: Unrecognized object type {} for {}",
                        object_type, &args.object
                    ),
                    128,
                )));
            }
        }
    }

    Ok(())
}

fn print_blob_content<R: Read>(reader: &mut R) {
    copy(reader, &mut stdout()).expect("Failed to print blob content");
}

fn print_tree_content<R: Read>(reader: &mut R) -> Result<(), Box<RGitError>> {
    let mut content = Vec::new();
    reader.read_to_end(&mut content).unwrap();

    let content = String::from_utf8_lossy(&content);
    for entry in content.split('\x00') {
        if !entry.is_empty() {
            let parts: Vec<&str> = entry.split_whitespace().collect();
            if parts.len() != 4 {
                return Err(Box::new(RGitError::new(
                    format!("fatal: Invalid tree entry format: {}", entry),
                    128,
                )));
            }
            let mode = parts[0];
            let object_type = parts[1];
            let hash = parts[2];
            let name = parts[3];
            println!("{:06} {} {}\t{}", mode, object_type, hash, name);
        }
    }

    Ok(())
}
