use super::error::RGitError;
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

pub fn hash_array_from_string(hash: &str) -> Result<[u8; 20]> {
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
    fn test_hash() {
        // use `echo -n "<content>" | sha1sum | awk '{print $1}'` to get the ground truth
        let content = "hello world";
        let result = hash(vec![content.as_bytes()].into_iter()).unwrap();

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
    fn test_hash_array_from_string() {
        let command = process::Command::new("sh")
            .arg("-c")
            .arg("echo -n \"hello world\" | sha1sum | awk '{print $1}'")
            .output()
            .unwrap();
        let ground_truth = String::from_utf8(command.stdout).unwrap();

        let result = hash_array_from_string(&ground_truth.trim()).unwrap();
        assert_eq!(hex::encode(result), ground_truth.trim());

        // test invalid hash
        let result = hash_array_from_string("invalid hash");
        assert!(result.is_err());
    }
}
