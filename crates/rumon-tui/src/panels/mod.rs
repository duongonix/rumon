//! TUI panels.

pub mod changes;
pub mod hooks;
pub mod logs;
pub mod plugins;
pub mod profiles;
pub mod remote;
pub mod status;

pub use changes::render_changes_panel;
pub use logs::render_logs_panel;
pub use remote::{RemotePanelNode, render_remote_panel};
pub use status::render_help_panel;
