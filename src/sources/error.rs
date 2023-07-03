use std::error::Error;
use std::fmt;


#[derive(Debug)]
pub struct SourceError {
    message: String,
}

impl SourceError {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl Error for SourceError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SourceError: {}", self.message)
    }
}
