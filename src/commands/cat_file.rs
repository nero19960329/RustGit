use super::super::error::RGitError;
use super::super::objects::rgit_object_from_hash;
use super::super::utils::get_rgit_dir;
use anyhow::Result;
use clap::{ArgGroup, Parser};
use std::env;
use std::io;

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

pub fn rgit_cat_file(args: &CatFileArgs) -> Result<()> {
    let rgit_dir = get_rgit_dir(env::current_dir()?.as_path())?;
    let mut hash_array = [0; 20];
    hex::decode_to_slice(&args.object, &mut hash_array).map_err(|_| {
        RGitError::new(
            format!("fatal: Not a valid object name {}", args.object),
            128,
        )
    })?;
    let object = rgit_object_from_hash(rgit_dir.as_path(), &hash_array)?;
    if args.t {
        println!("{}", object.header()?.object_type);
    } else if args.s {
        println!("{}", object.header()?.size);
    } else if args.p {
        object.serialize_object(rgit_dir.as_path(), &mut io::stdout())?;
    }

    Ok(())
}
