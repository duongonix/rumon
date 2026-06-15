//! Remote node session models.

/// Remote node connection state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteNodeState {
    /// Node is connected.
    Connected,
    /// Node is attempting to connect or authenticate.
    Connecting,
    /// Node is disconnected.
    Disconnected,
    /// Node failed.
    Failed(String),
}

/// A remote node known to the local client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteNode {
    /// Display name.
    pub name: String,
    /// Current connection state.
    pub state: RemoteNodeState,
    /// Recent logs received from the node.
    pub logs: Vec<RemoteLog>,
    /// Recent events received from the node.
    pub events: Vec<RemoteEvent>,
}

/// A remote log line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteLog {
    /// Log source such as stdout, stderr, or system.
    pub source: String,
    /// Log message.
    pub message: String,
}

/// A remote event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteEvent {
    /// Event kind.
    pub kind: String,
    /// Event message.
    pub message: String,
}

impl RemoteNode {
    /// Creates a connected remote node with empty history.
    #[must_use]
    pub fn connected(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: RemoteNodeState::Connected,
            logs: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Records a log line.
    pub fn push_log(&mut self, source: impl Into<String>, message: impl Into<String>) {
        self.logs.push(RemoteLog {
            source: source.into(),
            message: message.into(),
        });
    }

    /// Records an event.
    pub fn push_event(&mut self, kind: impl Into<String>, message: impl Into<String>) {
        self.events.push(RemoteEvent {
            kind: kind.into(),
            message: message.into(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{RemoteNode, RemoteNodeState};

    #[test]
    fn remote_node_tracks_state() {
        let node = RemoteNode {
            name: "node-a".to_string(),
            state: RemoteNodeState::Connected,
            logs: Vec::new(),
            events: Vec::new(),
        };

        assert_eq!(node.state, RemoteNodeState::Connected);
    }

    #[test]
    fn remote_node_records_logs() {
        let mut node = RemoteNode::connected("node-a");

        node.push_log("stdout", "ready");

        assert_eq!(node.logs.len(), 1);
    }
}
