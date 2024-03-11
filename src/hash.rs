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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    use std::process;

    fn get_expected_hash(input: &[u8]) -> [u8; 20] {
        let mut child = process::Command::new("sha1sum")
            .arg("-")
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()
            .unwrap();

        child.stdin.as_mut().unwrap().write_all(input).unwrap();
        let output = child.wait_with_output().unwrap();

        let hash_str = String::from_utf8(output.stdout).unwrap();
        let hash_bytes = hex::decode(&hash_str[..40]).unwrap();
        hash_bytes.try_into().unwrap()
    }

    #[test]
    fn test_hash_empty() {
        let input: Vec<u8> = vec![];
        let expected_hash = get_expected_hash(&[]);

        let result = hash(vec![input.as_slice()].into_iter()).unwrap();
        assert_eq!(result, expected_hash);
    }

    #[test]
    fn test_hash_single_reader() {
        let input = b"Hello, world!";
        let expected_hash = get_expected_hash(input);

        let result = hash(vec![input.as_ref()].into_iter()).unwrap();
        assert_eq!(result, expected_hash);
    }

    #[test]
    fn test_hash_multiple_readers() {
        let input = vec![b"Hello, ".to_vec(), b"world!".to_vec()];
        let expected_hash = get_expected_hash(b"Hello, world!");

        let result = hash(input.into_iter().map(io::Cursor::new)).unwrap();
        assert_eq!(result, expected_hash);
    }

    #[test]
    fn test_hash_error_propagation() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Other, "read failed"))
            }
        }

        let input = vec![FailingReader];
        let result = hash(input.into_iter());
        assert!(result.is_err());
    }
}
