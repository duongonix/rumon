//! Remote monitoring boundary for Rumon.

mod auth;
mod client;
mod error;
mod protocol;
mod server;
mod session;
mod telemetry;
mod transport;

pub use auth::RemoteToken;
pub use client::{RemoteClientConfig, RemoteClientReport, connect_remote};
pub use error::{RemoteError, RemoteResult};
pub use protocol::RemoteFrame;
pub use server::{RemoteAgentConfig, run_agent_once};
pub use session::{RemoteEvent, RemoteLog, RemoteNode, RemoteNodeState};
pub use telemetry::agent_startup_frames;
