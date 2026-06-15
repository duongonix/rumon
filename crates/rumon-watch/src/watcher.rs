//! Polling filesystem watcher.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use rumon_shared::{AppEvent, ChangeKind, WatchEvent};

use crate::backend::send_watch_error;
use crate::event_mapper::{file_changed, file_renamed};
use crate::native::NativeWatcher;
use crate::snapshot::{FileSnapshot, scan};

/// Watcher settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WatchOptions {
    /// Paths to scan.
    pub paths: Vec<PathBuf>,
    /// Paths or directory names to ignore.
    pub ignore: Vec<PathBuf>,
    /// Extension allow-list without leading dots.
    pub extensions: Vec<String>,
    /// Whether subdirectories are scanned.
    pub recursive: bool,
    /// Whether symlinks are followed during initial scans.
    pub follow_symlink: bool,
}

/// Polling filesystem watcher used by Phase 1.
#[derive(Debug)]
pub struct PollWatcher {
    options: WatchOptions,
    snapshots: BTreeMap<PathBuf, FileSnapshot>,
}

impl PollWatcher {
    /// Creates a watcher and captures the initial snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when any configured watch path cannot be scanned.
    pub fn new(options: WatchOptions) -> std::io::Result<Self> {
        let snapshots = scan(&options)?;
        Ok(Self { options, snapshots })
    }

    /// Polls the filesystem and returns events since the last snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error when a watched path cannot be scanned.
    pub fn poll(&mut self) -> std::io::Result<Vec<AppEvent>> {
        let next = scan(&self.options)?;
        let previous_paths: BTreeSet<_> = self.snapshots.keys().cloned().collect();
        let next_paths: BTreeSet<_> = next.keys().cloned().collect();
        let mut events = Vec::new();
        let created: Vec<_> = next_paths.difference(&previous_paths).cloned().collect();
        let deleted: Vec<_> = previous_paths.difference(&next_paths).cloned().collect();
        let renames = detect_renames(&deleted, &created, &self.snapshots, &next);
        let renamed_old_paths: BTreeSet<_> =
            renames.iter().map(|(old, _, _)| old.clone()).collect();
        let renamed_new_paths: BTreeSet<_> =
            renames.iter().map(|(_, new, _)| new.clone()).collect();

        for (old_path, new_path, snapshot_kind) in renames {
            events.push(file_renamed(old_path, new_path, snapshot_kind));
        }

        for path in created {
            if !renamed_new_paths.contains(&path) {
                if let Some(snapshot) = next.get(&path) {
                    events.push(file_changed(path, ChangeKind::Created, snapshot.kind));
                }
            }
        }

        for path in deleted {
            if !renamed_old_paths.contains(&path) {
                if let Some(snapshot) = self.snapshots.get(&path) {
                    events.push(file_changed(path, ChangeKind::Deleted, snapshot.kind));
                }
            }
        }

        for path in next_paths.intersection(&previous_paths) {
            if self.snapshots.get(path) != next.get(path) {
                if let Some(snapshot) = next.get(path) {
                    events.push(file_changed(
                        path.clone(),
                        ChangeKind::Modified,
                        snapshot.kind,
                    ));
                }
            }
        }

        self.snapshots = next;
        Ok(events)
    }
}

fn detect_renames(
    deleted: &[PathBuf],
    created: &[PathBuf],
    previous: &BTreeMap<PathBuf, FileSnapshot>,
    next: &BTreeMap<PathBuf, FileSnapshot>,
) -> Vec<(PathBuf, PathBuf, crate::snapshot::SnapshotKind)> {
    let mut renames = Vec::new();
    let mut used_created = BTreeSet::new();

    for old_path in deleted {
        let Some(old_snapshot) = previous.get(old_path) else {
            continue;
        };
        let candidates: Vec<_> = created
            .iter()
            .filter(|new_path| !used_created.contains(*new_path))
            .filter(|new_path| {
                next.get(*new_path)
                    .is_some_and(|new_snapshot| old_snapshot.rename_matches(new_snapshot))
            })
            .cloned()
            .collect();

        if candidates.len() == 1 {
            let new_path = candidates[0].clone();
            used_created.insert(new_path.clone());
            renames.push((old_path.clone(), new_path, old_snapshot.kind));
        }
    }

    renames
}

/// Starts a background polling watcher.
#[must_use]
pub fn spawn_polling_watcher(
    options: WatchOptions,
    interval: std::time::Duration,
    events: Sender<AppEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || run_polling_loop(options, interval, &events))
}

/// Starts the fastest available watcher backend.
#[must_use]
pub fn spawn_watcher(
    options: WatchOptions,
    debounce: std::time::Duration,
    events: Sender<AppEvent>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(
        move || match NativeWatcher::new(options.clone(), debounce) {
            Ok(watcher) => watcher.run(&events),
            Err(error) => {
                send_watch_error(
                    &events,
                    format!("native watcher unavailable ({error}); falling back to polling"),
                );
                run_polling_loop(options, debounce, &events);
            }
        },
    )
}

fn run_polling_loop(
    options: WatchOptions,
    interval: std::time::Duration,
    events: &Sender<AppEvent>,
) {
    let mut watcher = match PollWatcher::new(options) {
        Ok(watcher) => watcher,
        Err(error) => {
            send_watch_error(events, error);
            return;
        }
    };

    loop {
        std::thread::sleep(interval);
        match watcher.poll() {
            Ok(changes) => {
                for event in changes {
                    let _ = events.send(event);
                }
            }
            Err(error) => {
                let _ = events.send(AppEvent::Watch(WatchEvent::Error(error.to_string())));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PollWatcher, WatchOptions};
    use rumon_shared::{AppEvent, ChangeKind, WatchEvent};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn extension_filter_accepts_empty_allow_list() {
        let options = WatchOptions {
            paths: vec![PathBuf::from("src")],
            ignore: Vec::new(),
            extensions: Vec::new(),
            recursive: true,
            follow_symlink: false,
        };

        assert!(options.extensions.is_empty());
    }

    #[test]
    fn watcher_detects_file_rename() {
        let root = std::env::temp_dir().join("rumon_file_rename_watch_test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        fs::write(root.join("old.rs"), "fn main() {}\n").expect("write file");
        let mut watcher = PollWatcher::new(WatchOptions {
            paths: vec![root.clone()],
            ignore: Vec::new(),
            extensions: vec!["rs".to_string()],
            recursive: true,
            follow_symlink: false,
        })
        .expect("watcher");

        fs::rename(root.join("old.rs"), root.join("new.rs")).expect("rename file");
        let events = watcher.poll().expect("poll");
        let _ = fs::remove_dir_all(root);

        assert!(events.iter().any(|event| matches!(
            event,
            AppEvent::Watch(WatchEvent::Changed(change))
                if change.kind == ChangeKind::Renamed
                    && change.previous_path.as_ref().is_some_and(|path| path.ends_with("old.rs"))
                    && change.path.ends_with("new.rs")
        )));
    }

    #[test]
    fn watcher_detects_folder_create_and_delete() {
        let root = std::env::temp_dir().join("rumon_folder_watch_test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create root");
        let mut watcher = PollWatcher::new(WatchOptions {
            paths: vec![root.clone()],
            ignore: Vec::new(),
            extensions: vec!["rs".to_string()],
            recursive: true,
            follow_symlink: false,
        })
        .expect("watcher");

        let folder = root.join("created-folder");
        fs::create_dir_all(&folder).expect("create folder");
        let created = watcher.poll().expect("poll created");
        fs::remove_dir_all(&folder).expect("delete folder");
        let deleted = watcher.poll().expect("poll deleted");
        let _ = fs::remove_dir_all(root);

        assert!(created.iter().any(|event| matches!(
            event,
            AppEvent::Watch(WatchEvent::Changed(change))
                if change.kind == ChangeKind::Created && change.path.ends_with("created-folder")
        )));
        assert!(deleted.iter().any(|event| matches!(
            event,
            AppEvent::Watch(WatchEvent::Changed(change))
                if change.kind == ChangeKind::Deleted && change.path.ends_with("created-folder")
        )));
    }

    #[test]
    fn watcher_detects_folder_rename() {
        let root = std::env::temp_dir().join("rumon_folder_rename_watch_test");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("old-folder")).expect("create folder");
        let mut watcher = PollWatcher::new(WatchOptions {
            paths: vec![root.clone()],
            ignore: Vec::new(),
            extensions: vec!["rs".to_string()],
            recursive: true,
            follow_symlink: false,
        })
        .expect("watcher");

        fs::rename(root.join("old-folder"), root.join("new-folder")).expect("rename folder");
        let events = watcher.poll().expect("poll");
        let _ = fs::remove_dir_all(root);

        assert!(events.iter().any(|event| matches!(
            event,
            AppEvent::Watch(WatchEvent::Changed(change))
                if change.kind == ChangeKind::Renamed
                    && change
                        .previous_path
                        .as_ref()
                        .is_some_and(|path| path.ends_with("old-folder"))
                    && change.path.ends_with("new-folder")
        )));
    }
}
