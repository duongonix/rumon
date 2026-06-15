//! Shared models and event types used across Rumon crates.

mod events;
mod models;
mod path;
mod state;

pub use events::{AppEvent, ProcessEvent, WatchEvent};
pub use models::{ChangeDetail, ChangeKind, FileChange, LogEntry, LogKind, ProcessState};
pub use path::{cwd_relative_path, display_path};
pub use state::AppState;
