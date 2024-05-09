use crate::commands::{CatFileArgs, CheckIgnoreArgs, CommitArgs, HashObjectArgs, ReadTreeArgs};
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

    #[clap(name = "write-tree")]
    WriteTree,

    #[clap(name = "check-ignore")]
    CheckIgnore(CheckIgnoreArgs),

    #[clap(name = "read-tree")]
    ReadTree(ReadTreeArgs),

    #[clap(name = "commit")]
    Commit(CommitArgs),
}
