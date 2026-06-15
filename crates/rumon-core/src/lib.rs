//! Core runtime orchestration for Rumon.

mod change_detail;
mod error;
mod event_bus;
mod integrations;
mod orchestrator;
mod output;
mod state;

pub use error::{CoreError, CoreResult};
pub use event_bus::EventBus;
pub use integrations::{StreamFormat, run_api_server, run_ipc_daemon, run_watch_stream};
pub use orchestrator::{run, startup_message};
pub use state::Runtime;
