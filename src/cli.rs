use super::commands::{CatFileArgs, HashObjectArgs};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, author, about)]
pub struct RustGitArgs {
    #[clap(subcommand)]
    pub command: Option<RustGitSubCommands>,
}

#[derive(Subcommand, Debug)]
pub enum RustGitSubCommands {
    #[clap(name = "init")]
    Init,

    #[clap(name = "hash-object")]
    HashObject(HashObjectArgs),

    #[clap(name = "cat-file")]
    CatFile(CatFileArgs),
}