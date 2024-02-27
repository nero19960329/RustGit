use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use std::process;

#[derive(Parser, Debug)]
#[clap(version, author, about)]
struct RustGitArgs {
    #[clap(subcommand)]
    command: Option<RustGitSubCommands>,
}

#[derive(Subcommand, Debug)]
enum RustGitSubCommands {
    #[clap(name = "init")]
    Init,

    #[clap(name = "hash-object")]
    HashObject(HashObjectArgs),

    #[clap(name = "cat-file")]
    CatFile(CatFileArgs),
}

/// Compute object ID
#[derive(Parser, Debug)]
struct HashObjectArgs {
    /// Specify the type
    #[arg(name = "type", default_value = "blob", short)]
    type_: Option<String>,

    /// Actually write the object into the object database
    #[arg(name = "write", short)]
    write: bool,

    file: String,
}

/// Provide content for repository objects
#[derive(Parser, Debug)]
struct CatFileArgs {
    /// Instead of the content, show the object type identified by <object>
    #[arg(name = "t", short)]
    t: bool,

    /// Instead of the content, show the object size identified by <object>
    #[arg(short)]
    s: bool,

    /// Pretty-print the contents of <object> based on its type
    #[arg(short)]
    p: bool,

    /// The name of the object to show
    object: String,
}

fn rgit_init() {
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
}

fn rgit_hash_object(args: &HashObjectArgs) {
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
        args.type_.as_ref().unwrap().as_bytes(),
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

fn rgit_cat_file(args: &CatFileArgs) {
    let rgit_dir = env::current_dir().unwrap().join(".rgit");
    if fs::metadata(&rgit_dir).is_err() {
        eprintln!("fatal: not an rgit repository (or any of the parent directories): .rgit");
        process::exit(128);
    }

    let object = &rgit_dir.join("objects").join(&args.object);
    if fs::metadata(&object).is_err() {
        eprintln!("fatal: Not a valid object name {}", &args.object);
        process::exit(128);
    }

    let object_file = File::open(&object).unwrap();
    let mut reader = io::BufReader::new(object_file);

    let mut header = Vec::new();
    reader.read_until(b'\x00', &mut header).unwrap();
    let header = String::from_utf8(header).unwrap();
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
}

fn main() {
    let args = RustGitArgs::parse();

    match &args.command {
        Some(RustGitSubCommands::Init) => rgit_init(),
        Some(RustGitSubCommands::HashObject(args)) => rgit_hash_object(args),
        Some(RustGitSubCommands::CatFile(args)) => rgit_cat_file(args),
        None => {
            eprintln!("fatal: no command provided");
            process::exit(128);
        }
    }
}
