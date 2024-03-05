mod cli;
mod commands;
mod error;
mod hash;
mod ignore;
mod utils;

use clap::Parser;
use cli::{RustGitArgs, RustGitSubCommands};
use commands::{rgit_cat_file, rgit_hash_object, rgit_init, rgit_write_tree};
use error::RGitError;
use std::process;

fn main() {
    let args = RustGitArgs::parse();

    let result = match &args.command {
        Some(RustGitSubCommands::Init) => rgit_init(),
        Some(RustGitSubCommands::HashObject(args)) => rgit_hash_object(args),
        Some(RustGitSubCommands::CatFile(args)) => rgit_cat_file(args),
        Some(RustGitSubCommands::WriteTree) => rgit_write_tree(),
        None => Err(Box::new(RGitError::new(
            "fatal: no command provided".to_string(),
            128,
        ))),
    };

    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(e.exit_code.into());
    }
}
