//! Remote telemetry frame helpers.

use crate::protocol::RemoteFrame;

/// Builds startup frames sent by a remote agent.
#[must_use]
pub fn agent_startup_frames(node_name: &str) -> Vec<RemoteFrame> {
    vec![
        RemoteFrame::State {
            node: node_name.to_string(),
            state: "connected".to_string(),
        },
        RemoteFrame::Event {
            kind: "agent".to_string(),
            message: "remote agent connected".to_string(),
        },
        RemoteFrame::Log {
            source: "system".to_string(),
            message: "remote telemetry stream ready".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::agent_startup_frames;

    #[test]
    fn startup_frames_include_state() {
        let frames = agent_startup_frames("node-a");

        assert_eq!(frames.len(), 3);
    }
}
