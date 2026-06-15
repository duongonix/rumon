//! Configuration errors.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Convenient result type for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Configuration error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    /// Creates a configuration error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ConfigError {}
