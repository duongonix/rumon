//! Line-oriented remote monitor protocol.

use crate::error::{RemoteError, RemoteResult};

/// Remote protocol frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteFrame {
    /// Client hello with node name.
    Hello {
        /// Node display name.
        node: String,
    },
    /// Authentication token.
    Auth {
        /// Shared authentication token.
        token: String,
    },
    /// Authentication accepted.
    AuthOk,
    /// Authentication rejected.
    AuthFailed {
        /// Rejection reason.
        reason: String,
    },
    /// Remote log line.
    Log {
        /// Log source.
        source: String,
        /// Log message.
        message: String,
    },
    /// Remote event line.
    Event {
        /// Event kind.
        kind: String,
        /// Event message.
        message: String,
    },
    /// Remote node state update.
    State {
        /// Node display name.
        node: String,
        /// State label.
        state: String,
    },
    /// Keepalive ping.
    Ping,
    /// Graceful disconnect.
    Disconnect {
        /// Disconnect reason.
        reason: String,
    },
    /// Error frame.
    Error {
        /// Error message.
        message: String,
    },
}

impl RemoteFrame {
    /// Encodes this frame as one protocol line.
    #[must_use]
    pub fn encode(&self) -> String {
        match self {
            Self::Hello { node } => format!("HELLO {}", escape(node)),
            Self::Auth { token } => format!("AUTH {}", escape(token)),
            Self::AuthOk => "AUTH_OK".to_string(),
            Self::AuthFailed { reason } => format!("AUTH_FAILED {}", escape(reason)),
            Self::Log { source, message } => {
                format!("LOG {} {}", escape(source), escape(message))
            }
            Self::Event { kind, message } => {
                format!("EVENT {} {}", escape(kind), escape(message))
            }
            Self::State { node, state } => {
                format!("STATE {} {}", escape(node), escape(state))
            }
            Self::Ping => "PING".to_string(),
            Self::Disconnect { reason } => format!("DISCONNECT {}", escape(reason)),
            Self::Error { message } => format!("ERROR {}", escape(message)),
        }
    }

    /// Decodes one protocol line into a frame.
    ///
    /// # Errors
    ///
    /// Returns an error when the frame is malformed or unknown.
    pub fn decode(line: &str) -> RemoteResult<Self> {
        let mut parts = line.trim_end().splitn(3, ' ');
        let command = parts.next().unwrap_or_default();
        match command {
            "HELLO" => Ok(Self::Hello {
                node: unescape(parts.next().unwrap_or_default()),
            }),
            "AUTH" => Ok(Self::Auth {
                token: unescape(parts.next().unwrap_or_default()),
            }),
            "AUTH_OK" => Ok(Self::AuthOk),
            "AUTH_FAILED" => Ok(Self::AuthFailed {
                reason: unescape(parts.next().unwrap_or_default()),
            }),
            "LOG" => Ok(Self::Log {
                source: unescape(parts.next().unwrap_or_default()),
                message: unescape(parts.next().unwrap_or_default()),
            }),
            "EVENT" => Ok(Self::Event {
                kind: unescape(parts.next().unwrap_or_default()),
                message: unescape(parts.next().unwrap_or_default()),
            }),
            "STATE" => Ok(Self::State {
                node: unescape(parts.next().unwrap_or_default()),
                state: unescape(parts.next().unwrap_or_default()),
            }),
            "PING" => Ok(Self::Ping),
            "DISCONNECT" => Ok(Self::Disconnect {
                reason: unescape(parts.next().unwrap_or_default()),
            }),
            "ERROR" => Ok(Self::Error {
                message: unescape(parts.next().unwrap_or_default()),
            }),
            _ => Err(RemoteError::new(format!("unknown remote frame: {command}"))),
        }
    }
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace(' ', "\\s")
}

fn unescape(value: &str) -> String {
    let mut output = String::new();
    let mut escaping = false;
    for character in value.chars() {
        if escaping {
            match character {
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                's' => output.push(' '),
                '\\' => output.push('\\'),
                other => output.push(other),
            }
            escaping = false;
        } else if character == '\\' {
            escaping = true;
        } else {
            output.push(character);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::RemoteFrame;

    #[test]
    fn round_trips_log_frame() {
        let frame = RemoteFrame::Log {
            source: "stdout".to_string(),
            message: "hello remote".to_string(),
        };

        assert_eq!(
            RemoteFrame::decode(&frame.encode()).expect("decode frame"),
            frame
        );
    }
}
