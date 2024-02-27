use clap::{Parser, Subcommand};
use sha1::{Digest, Sha1};
use std::env;
use std::fs;

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
}

/// Compute object ID
#[derive(Parser, Debug)]
struct HashObjectArgs {
    file: String,
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
    let file = env::current_dir().unwrap().join(&args.file);
    let data = fs::read(&file).unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let hash_result = hasher.finalize();
    let hash = format!("{:x}", hash_result);
    let path = format!(".rgit/objects/{}", &hash);
    fs::write(&path, &data).unwrap();
    println!("{}", hash);
}

fn main() {
    let args = RustGitArgs::parse();

    match &args.command {
        Some(RustGitSubCommands::Init) => rgit_init(),
        Some(RustGitSubCommands::HashObject(args)) => rgit_hash_object(args),
        None => {
            println!("unknown command");
        }
    }
}
