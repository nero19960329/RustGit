use anyhow::Result;
use sha1::{Digest, Sha1};
use std::io::{self, Read};

pub fn hash(readers: impl Iterator<Item = impl Read>) -> Result<String> {
    let mut hasher = Sha1::new();
    for mut reader in readers {
        io::copy(&mut reader, &mut hasher)?;
    }
    let result = format!("{:x}", hasher.finalize());
    Ok(result)
}
