//! Profile loading errors.

use std::fmt::{self, Display, Formatter};

/// Result type used by profile operations.
pub type ProfileResult<T> = Result<T, ProfileError>;

/// Error returned when a profile cannot be resolved.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileError {
    message: String,
}

impl ProfileError {
    /// Creates a profile error with a human-readable message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for ProfileError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ProfileError {}
