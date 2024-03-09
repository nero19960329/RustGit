use anyhow::Result;
use sha1::{Digest, Sha1};
use std::io::{self, Read};

pub fn hash(readers: impl Iterator<Item = impl Read>) -> Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    for mut reader in readers {
        io::copy(&mut reader, &mut hasher)?;
    }
    let result = hasher.finalize();
    Ok(result.into())
}
