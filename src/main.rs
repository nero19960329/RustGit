mod cli;
mod commands;
mod error;
mod hash;
mod ignore;
mod objects;
mod utils;

use clap::Parser;
use cli::{RustGitArgs, RustGitSubCommands};
use commands::{
    rgit_cat_file, rgit_check_ignore, rgit_commit, rgit_hash_object, rgit_init, rgit_read_tree,
    rgit_write_tree,
};
use error::RGitError;
use std::process;

fn main() {
    let args = RustGitArgs::parse();

    let result = match &args.command {
        Some(RustGitSubCommands::Init) => rgit_init(),
        Some(RustGitSubCommands::HashObject(args)) => rgit_hash_object(args),
        Some(RustGitSubCommands::CatFile(args)) => rgit_cat_file(args),
        Some(RustGitSubCommands::WriteTree) => rgit_write_tree(),
        Some(RustGitSubCommands::CheckIgnore(args)) => rgit_check_ignore(args),
        Some(RustGitSubCommands::ReadTree(args)) => rgit_read_tree(args),
        Some(RustGitSubCommands::Commit(args)) => rgit_commit(args),
        None => Err(RGitError::new(
            "fatal: no command provided".to_string(),
            128,
        )),
    };

    if let Err(err) = result {
        match err.downcast_ref::<RGitError>() {
            Some(rgit_err) => {
                eprintln!("{}", rgit_err.message);
                process::exit(rgit_err.exit_code.into());
            }
            None => {
                eprintln!("An unexpected error occurred: {}", err);
                process::exit(1);
            }
        }
    }
    process::exit(result.unwrap().into());
}
