//! Filesystem watching for Rumon.

mod backend;
mod event_mapper;
mod filter;
mod native;
mod notify_mapper;
mod snapshot;
mod watcher;

pub use event_mapper::file_changed;
pub use snapshot::FileSnapshot;
pub use watcher::{PollWatcher, WatchOptions, spawn_polling_watcher, spawn_watcher};
