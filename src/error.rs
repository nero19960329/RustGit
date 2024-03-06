use anyhow::Error;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub struct RGitError {
    pub message: String,
    pub exit_code: u8,
}

impl RGitError {
    pub fn new(message: String, exit_code: u8) -> Error {
        Error::msg(Self { message, exit_code })
    }
}

impl Display for RGitError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message,)
    }
}
