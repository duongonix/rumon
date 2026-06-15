//! Shared watcher backend helpers.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use rumon_shared::{AppEvent, WatchEvent};

use crate::filter::{extension_allowed, should_ignore};
use crate::watcher::WatchOptions;

/// Sends a watcher error into the runtime event bus.
pub(crate) fn send_watch_error(events: &Sender<AppEvent>, error: impl std::fmt::Display) {
    let _ = events.send(AppEvent::Watch(WatchEvent::Error(error.to_string())));
}

/// Returns whether a concrete path should be allowed through the watch filters.
#[must_use]
pub(crate) fn path_allowed(path: &Path, options: &WatchOptions) -> bool {
    !should_ignore(path, &options.ignore)
        && (path_is_dir(path) || extension_allowed(path, &options.extensions))
}

/// Returns configured watch roots that currently exist.
#[must_use]
pub(crate) fn existing_roots(options: &WatchOptions) -> Vec<PathBuf> {
    options
        .paths
        .iter()
        .filter(|path| path.exists())
        .cloned()
        .collect()
}

fn path_is_dir(path: &Path) -> bool {
    path.metadata().is_ok_and(|metadata| metadata.is_dir())
}
