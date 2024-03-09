use super::super::objects::rgit_object_from_hash;
use anyhow::Result;
use clap::{ArgGroup, Parser};

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
    let object = rgit_object_from_hash(&args.object)?;
    if args.t {
        println!("{}", object.header()?.object_type);
    } else if args.s {
        println!("{}", object.header()?.size);
    } else if args.p {
        object.print_object()?;
    }

    Ok(())
}
