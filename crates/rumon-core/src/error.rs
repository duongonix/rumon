//! Core runtime errors.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Convenient result type for core runtime operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Runtime error produced by the core orchestrator.
#[derive(Debug)]
pub struct CoreError {
    message: String,
}

impl CoreError {
    /// Creates a core runtime error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for CoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CoreError {}
