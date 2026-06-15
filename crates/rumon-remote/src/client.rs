//! Remote monitor client.

use std::net::TcpStream;

use crate::auth::RemoteToken;
use crate::error::{RemoteError, RemoteResult};
use crate::protocol::RemoteFrame;
use crate::session::{RemoteNode, RemoteNodeState};
use crate::transport::{connect, frame_reader, read_frame, send_frame};

/// Remote client connection settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteClientConfig {
    /// Server address.
    pub address: String,
    /// Local node display name.
    pub node_name: String,
    /// Authentication token.
    pub token: RemoteToken,
    /// Maximum frames to read before returning. `None` reads until disconnect.
    pub max_frames: Option<usize>,
}

/// Remote client result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteClientReport {
    /// Remote node session state.
    pub node: RemoteNode,
    /// Number of frames received after authentication.
    pub frames_received: usize,
}

/// Connects to a remote agent and receives monitor frames.
///
/// # Errors
///
/// Returns an error when connection, authentication, or frame processing fails.
pub fn connect_remote(config: &RemoteClientConfig) -> RemoteResult<RemoteClientReport> {
    let mut stream = connect(config.address.as_str())?;
    authenticate_client(&mut stream, config)?;

    let mut reader = frame_reader(&stream)?;
    let mut node = RemoteNode::connected(config.node_name.clone());
    let mut frames_received = 0;

    loop {
        if config
            .max_frames
            .is_some_and(|max_frames| frames_received >= max_frames)
        {
            break;
        }
        let Some(frame) = read_frame(&mut reader)? else {
            node.state = RemoteNodeState::Disconnected;
            break;
        };
        frames_received += 1;
        apply_frame(&mut node, frame);
    }

    Ok(RemoteClientReport {
        node,
        frames_received,
    })
}

fn authenticate_client(stream: &mut TcpStream, config: &RemoteClientConfig) -> RemoteResult<()> {
    send_frame(
        stream,
        &RemoteFrame::Hello {
            node: config.node_name.clone(),
        },
    )?;
    send_frame(
        stream,
        &RemoteFrame::Auth {
            token: config.token.expose().to_string(),
        },
    )?;

    let mut reader = frame_reader(stream)?;
    match read_frame(&mut reader)? {
        Some(RemoteFrame::AuthOk) => Ok(()),
        Some(RemoteFrame::AuthFailed { reason }) => {
            Err(RemoteError::new(format!("remote auth failed: {reason}")))
        }
        Some(frame) => Err(RemoteError::new(format!(
            "unexpected auth response: {}",
            frame.encode()
        ))),
        None => Err(RemoteError::new("remote disconnected during auth")),
    }
}

fn apply_frame(node: &mut RemoteNode, frame: RemoteFrame) {
    match frame {
        RemoteFrame::Log { source, message } => node.push_log(source, message),
        RemoteFrame::Event { kind, message } => node.push_event(kind, message),
        RemoteFrame::State { node: name, state } => {
            node.name = name;
            node.state = match state.as_str() {
                "connected" => RemoteNodeState::Connected,
                "disconnected" => RemoteNodeState::Disconnected,
                other => RemoteNodeState::Failed(other.to_string()),
            };
        }
        RemoteFrame::Disconnect { reason } => {
            node.push_event("disconnect", reason);
            node.state = RemoteNodeState::Disconnected;
        }
        RemoteFrame::Error { message } => {
            node.state = RemoteNodeState::Failed(message);
        }
        RemoteFrame::Ping
        | RemoteFrame::Hello { .. }
        | RemoteFrame::Auth { .. }
        | RemoteFrame::AuthOk
        | RemoteFrame::AuthFailed { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{RemoteClientConfig, connect_remote};
    use crate::RemoteAgentConfig;
    use crate::auth::RemoteToken;
    use crate::server::run_agent_once;
    use std::net::TcpListener;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn client_receives_agent_frames() {
        let token = RemoteToken::new("secret").expect("token");
        let address = available_loopback_address();
        let agent_token = token.clone();
        let agent_address = address.clone();
        let handle = thread::spawn(move || {
            run_agent_once(&RemoteAgentConfig {
                address: agent_address,
                node_name: "agent-a".to_string(),
                token: agent_token,
            })
            .expect("agent should run");
        });
        thread::sleep(Duration::from_millis(100));

        let report = connect_remote(&RemoteClientConfig {
            address,
            node_name: "client-a".to_string(),
            token,
            max_frames: None,
        })
        .expect("client should connect");

        handle.join().expect("agent thread should finish");

        assert!(report.frames_received >= 1);
        assert!(!report.node.events.is_empty());
    }

    fn available_loopback_address() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port");
        let address = listener.local_addr().expect("local address");
        address.to_string()
    }
}
