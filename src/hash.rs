use crate::error::RGitError;
use anyhow::Result;
use sha1::{self, Digest};
use std::io::Read;

pub trait Hasher {
    fn update(&mut self, data: &[u8]);
    fn finalize(self) -> [u8; 20];
}

pub struct Sha1 {
    inner: sha1::Sha1,
}

impl Sha1 {
    pub fn new() -> Self {
        Self {
            inner: sha1::Sha1::new(),
        }
    }
}

impl Hasher for Sha1 {
    fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    fn finalize(self) -> [u8; 20] {
        self.inner.finalize().into()
    }
}

pub fn hash_object<R: Read>(mut reader: R) -> Result<[u8; 20]> {
    let mut hasher = Sha1::new();
    let mut buffer = [0; 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hasher.finalize())
}

pub fn hash_array_from_str(hash: &str) -> Result<[u8; 20]> {
    let mut hash_array = [0u8; 20];
    hex::decode_to_slice(hash, &mut hash_array)
        .map_err(|_| RGitError::new("fatal: Not a valid object name".to_string(), 128))?;
    Ok(hash_array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    #[test]
    fn test_hash_object() {
        // use `echo -n "<content>" | sha1sum | awk '{print $1}'` to get the ground truth
        let content = "hello world";
        let result = hash_object(content.as_bytes()).unwrap();

        let command = process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "echo -n \"{}\" | sha1sum | awk '{{print $1}}'",
                content
            ))
            .output()
            .unwrap();
        let ground_truth = String::from_utf8(command.stdout).unwrap();

        assert_eq!(hex::encode(result), ground_truth.trim());
    }

    #[test]
    fn test_hash_array_from_str() {
        let command = process::Command::new("sh")
            .arg("-c")
            .arg("echo -n \"hello world\" | sha1sum | awk '{print $1}'")
            .output()
            .unwrap();
        let ground_truth = String::from_utf8(command.stdout).unwrap();

        let result = hash_array_from_str(&ground_truth.trim()).unwrap();
        assert_eq!(hex::encode(result), ground_truth.trim());

        // test invalid hash
        let result = hash_array_from_str("invalid hash");
        assert!(result.is_err());
    }
}
