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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgit_error_new() {
        let message = "An error occurred".to_string();
        let exit_code = 1;

        let error = RGitError::new(message.clone(), exit_code);

        assert!(error.is::<RGitError>());
        let rgit_error = error.downcast_ref::<RGitError>().unwrap();
        assert_eq!(rgit_error.message, message);
        assert_eq!(rgit_error.exit_code, exit_code);
    }

    #[test]
    fn test_rgit_error_display() {
        let message = "An error occurred".to_string();
        let exit_code = 2;

        let error = RGitError::new(message.clone(), exit_code);

        assert_eq!(format!("{}", error), message);
    }
}
