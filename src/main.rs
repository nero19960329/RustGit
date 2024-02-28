mod cli;
mod commands;

use clap::Parser;
use cli::{RustGitArgs, RustGitSubCommands};
use commands::{rgit_cat_file, rgit_hash_object, rgit_init};
use std::process;

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
