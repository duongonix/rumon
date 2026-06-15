//! Watch event mapping helpers.

use std::path::PathBuf;

use rumon_shared::{AppEvent, ChangeKind, FileChange, WatchEvent, cwd_relative_path};

use crate::snapshot::SnapshotKind;

/// Builds a watch event from a path and change kind.
#[must_use]
pub fn file_changed(
    path: impl Into<PathBuf>,
    kind: ChangeKind,
    snapshot_kind: SnapshotKind,
) -> AppEvent {
    let path = path.into();
    AppEvent::Watch(WatchEvent::Changed(FileChange {
        path: cwd_relative_path(&path),
        previous_path: None,
        kind,
        is_directory: snapshot_kind == SnapshotKind::Directory,
        detail: None,
    }))
}

/// Builds a rename watch event from old and new paths.
#[must_use]
pub fn file_renamed(
    previous_path: impl Into<PathBuf>,
    path: impl Into<PathBuf>,
    snapshot_kind: SnapshotKind,
) -> AppEvent {
    let previous_path = previous_path.into();
    let path = path.into();
    AppEvent::Watch(WatchEvent::Changed(FileChange {
        path: cwd_relative_path(&path),
        previous_path: Some(cwd_relative_path(&previous_path)),
        kind: ChangeKind::Renamed,
        is_directory: snapshot_kind == SnapshotKind::Directory,
        detail: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::{file_changed, file_renamed};
    use crate::snapshot::SnapshotKind;
    use rumon_shared::{AppEvent, ChangeKind, WatchEvent};
    use std::path::PathBuf;

    #[test]
    fn creates_watch_event() {
        let event = file_changed("src/main.rs", ChangeKind::Modified, SnapshotKind::File);

        assert_eq!(
            event,
            AppEvent::Watch(WatchEvent::Changed(rumon_shared::FileChange {
                path: PathBuf::from("src/main.rs"),
                previous_path: None,
                kind: ChangeKind::Modified,
                is_directory: false,
                detail: None,
            }))
        );
    }

    #[test]
    fn creates_rename_event() {
        let event = file_renamed("src/old.rs", "src/new.rs", SnapshotKind::File);

        assert_eq!(
            event,
            AppEvent::Watch(WatchEvent::Changed(rumon_shared::FileChange {
                path: PathBuf::from("src/new.rs"),
                previous_path: Some(PathBuf::from("src/old.rs")),
                kind: ChangeKind::Renamed,
                is_directory: false,
                detail: None,
            }))
        );
    }
}
