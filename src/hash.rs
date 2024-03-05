use super::error::RGitError;
use super::utils::get_rgit_dir;
use sha1::{Digest, Sha1};
use std::fs;

pub fn hash_object(data: &[u8], object_type: &str, write: bool) -> Result<String, RGitError> {
    let mut hasher = Sha1::new();
    let header = format!("{} {}\x00", object_type, data.len());
    hasher.update(header.as_bytes());
    hasher.update(data);
    let hash = format!("{:x}", hasher.finalize());

    if write {
        let rgit_dir = get_rgit_dir()?;
        let object_path = rgit_dir.join("objects").join(&hash);
        fs::create_dir_all(object_path.parent().unwrap()).unwrap();
        fs::write(object_path, [header.as_bytes(), data].concat()).unwrap();
    }

    Ok(hash)
}
