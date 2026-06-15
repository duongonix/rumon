//! Remote authentication helpers.

use crate::error::{RemoteError, RemoteResult};

/// Remote authentication token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteToken(String);

impl RemoteToken {
    /// Creates a token.
    ///
    /// # Errors
    ///
    /// Returns an error when the token is empty.
    pub fn new(value: impl Into<String>) -> RemoteResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(RemoteError::new("remote token must not be empty"));
        }
        Ok(Self(value))
    }

    /// Returns the token as a string slice.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }

    /// Returns whether the supplied token matches this token.
    #[must_use]
    pub fn matches(&self, supplied: &str) -> bool {
        self.0 == supplied
    }
}

#[cfg(test)]
mod tests {
    use super::RemoteToken;

    #[test]
    fn rejects_empty_tokens() {
        assert!(RemoteToken::new("").is_err());
    }

    #[test]
    fn matches_same_token() {
        let token = RemoteToken::new("secret").expect("token");

        assert!(token.matches("secret"));
        assert!(!token.matches("nope"));
    }
}
