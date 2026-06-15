//! Path formatting helpers shared by runtime-facing crates.

use std::path::{Path, PathBuf};

/// Returns a path relative to the current Rumon working directory when possible.
#[must_use]
pub fn cwd_relative_path(path: &Path) -> PathBuf {
    let Ok(current_dir) = std::env::current_dir() else {
        return path.to_path_buf();
    };

    path.strip_prefix(&current_dir)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}

/// Formats a path for UI, logs, and rule contexts using `/` separators.
#[must_use]
pub fn display_path(path: &Path) -> String {
    cwd_relative_path(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::{cwd_relative_path, display_path};

    #[test]
    fn keeps_relative_paths_relative() {
        assert_eq!(display_path("src/main.rs".as_ref()), "src/main.rs");
    }

    #[test]
    fn strips_current_dir_from_absolute_paths() {
        let absolute = std::env::current_dir()
            .expect("cwd")
            .join("src")
            .join("main.rs");

        assert_eq!(
            cwd_relative_path(&absolute),
            std::path::PathBuf::from("src/main.rs")
        );
    }
}
