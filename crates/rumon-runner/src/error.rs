//! Runner errors.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// Convenient result type for runner operations.
pub type RunnerResult<T> = Result<T, RunnerError>;

/// Runner error.
#[derive(Debug)]
pub struct RunnerError {
    message: String,
}

impl RunnerError {
    /// Creates a runner error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for RunnerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for RunnerError {}

/// Converts an I/O error to a runner error.
#[must_use]
pub fn to_runner_error(error: &std::io::Error) -> RunnerError {
    RunnerError::new(error.to_string())
}
