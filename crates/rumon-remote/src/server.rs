//! Remote monitor agent/server.

use std::net::TcpStream;

use crate::auth::RemoteToken;
use crate::error::{RemoteError, RemoteResult};
use crate::protocol::RemoteFrame;
use crate::telemetry::agent_startup_frames;
use crate::transport::{bind, frame_reader, read_frame, send_frame};

/// Remote agent settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteAgentConfig {
    /// Bind address.
    pub address: String,
    /// Agent node name.
    pub node_name: String,
    /// Authentication token.
    pub token: RemoteToken,
}

/// Runs a remote agent and serves one client connection.
///
/// # Errors
///
/// Returns an error when binding, authentication, or frame sending fails.
pub fn run_agent_once(config: &RemoteAgentConfig) -> RemoteResult<()> {
    let listener = bind(&config.address)?;
    let (mut stream, _) = listener.accept()?;
    authenticate_agent(&mut stream, config)?;
    for frame in agent_startup_frames(&config.node_name) {
        send_frame(&mut stream, &frame)?;
    }
    send_frame(
        &mut stream,
        &RemoteFrame::Disconnect {
            reason: "agent session complete".to_string(),
        },
    )?;
    Ok(())
}

fn authenticate_agent(stream: &mut TcpStream, config: &RemoteAgentConfig) -> RemoteResult<()> {
    let mut reader = frame_reader(stream)?;
    let hello = read_frame(&mut reader)?;
    if !matches!(hello, Some(RemoteFrame::Hello { .. })) {
        send_frame(
            stream,
            &RemoteFrame::AuthFailed {
                reason: "expected hello".to_string(),
            },
        )?;
        return Err(RemoteError::new("expected remote hello"));
    }

    match read_frame(&mut reader)? {
        Some(RemoteFrame::Auth { token }) if config.token.matches(&token) => {
            send_frame(stream, &RemoteFrame::AuthOk)?;
            Ok(())
        }
        Some(RemoteFrame::Auth { .. }) => {
            send_frame(
                stream,
                &RemoteFrame::AuthFailed {
                    reason: "invalid token".to_string(),
                },
            )?;
            Err(RemoteError::new("invalid remote token"))
        }
        _ => {
            send_frame(
                stream,
                &RemoteFrame::AuthFailed {
                    reason: "expected auth".to_string(),
                },
            )?;
            Err(RemoteError::new("expected remote auth"))
        }
    }
}
