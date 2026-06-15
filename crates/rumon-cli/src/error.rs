//! CLI errors.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// CLI parsing error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliError {
    message: String,
}

impl CliError {
    /// Creates a CLI error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for CliError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CliError {}
