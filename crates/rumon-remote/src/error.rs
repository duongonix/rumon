//! Remote monitor errors.

use std::fmt::{self, Display, Formatter};

/// Result type used by remote monitor operations.
pub type RemoteResult<T> = Result<T, RemoteError>;

/// Error returned by remote monitor operations.
#[derive(Debug)]
pub struct RemoteError {
    message: String,
}

impl RemoteError {
    /// Creates a remote error with a human-readable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for RemoteError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RemoteError {}

impl From<std::io::Error> for RemoteError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}
