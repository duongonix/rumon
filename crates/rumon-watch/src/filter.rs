//! Filesystem watch filters.

use std::path::{Path, PathBuf};

/// Returns true when a path should be ignored.
#[must_use]
pub fn should_ignore(path: &Path, ignore: &[PathBuf]) -> bool {
    ignore.iter().any(|ignored| {
        path == ignored
            || path.starts_with(ignored)
            || ignored.file_name().is_some_and(|name| {
                path.components()
                    .any(|component| component.as_os_str() == name)
            })
    })
}

/// Returns true when an extension is accepted by the configured allow-list.
#[must_use]
pub fn extension_allowed(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.iter().any(|allowed| allowed == extension))
}
