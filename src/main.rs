use clap::{Parser, Subcommand};
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
}

fn rgit_init() {
    let rgit_dir = env::current_dir().unwrap().join(".rgit");
    if fs::metadata(".rgit").is_err() {
        fs::create_dir(".rgit").unwrap();

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

fn main() {
    let args = RustGitArgs::parse();

    match &args.command {
        Some(RustGitSubCommands::Init) => rgit_init(),
        None => {
            println!("unknown command");
        }
    }
}
