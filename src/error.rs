use std::fmt::{Display, Formatter, Result};

pub const NOT_RGIT_REPOSITORY: &str =
    "fatal: not a rgit repository (or any of the parent directories): .rgit";

#[derive(Debug)]
pub struct RGitError {
    message: String,
    pub exit_code: u8,
}

impl RGitError {
    pub fn new(message: String, exit_code: u8) -> Self {
        RGitError { message, exit_code }
    }
}

impl Display for RGitError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message,)
    }
}
