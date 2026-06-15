//! Mapping from notify events to Rumon watch events.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use notify::event::{ModifyKind, RenameMode};
use notify::{Event, EventKind};
use rumon_shared::{AppEvent, ChangeKind};

use crate::backend::path_allowed;
use crate::event_mapper::{file_changed, file_renamed};
use crate::snapshot::{FileSnapshot, SnapshotKind};
use crate::watcher::WatchOptions;

/// Tracks file/folder kind information so remove and rename events can stay precise.
#[derive(Debug)]
pub(crate) struct KindCache {
    entries: BTreeMap<PathBuf, SnapshotKind>,
}

impl KindCache {
    /// Creates a kind cache from a filesystem snapshot.
    #[must_use]
    pub(crate) fn new(snapshots: &BTreeMap<PathBuf, FileSnapshot>) -> Self {
        Self {
            entries: snapshots
                .iter()
                .map(|(path, snapshot)| (path.clone(), snapshot.kind))
                .collect(),
        }
    }

    /// Updates the cache and maps a notify event into Rumon events.
    pub(crate) fn map_event(&mut self, event: &Event, options: &WatchOptions) -> Vec<AppEvent> {
        let mapped = map_event(event, options, self);
        for runtime_event in &mapped {
            self.apply_runtime_event(runtime_event);
        }
        mapped
    }

    fn apply_runtime_event(&mut self, event: &AppEvent) {
        let AppEvent::Watch(rumon_shared::WatchEvent::Changed(change)) = event else {
            return;
        };

        match change.kind {
            ChangeKind::Created | ChangeKind::Modified => {
                let kind = if change.is_directory {
                    SnapshotKind::Directory
                } else {
                    SnapshotKind::File
                };
                self.entries.insert(change.path.clone(), kind);
            }
            ChangeKind::Deleted => {
                self.entries.remove(&change.path);
            }
            ChangeKind::Renamed => {
                if let Some(previous_path) = &change.previous_path {
                    self.entries.remove(previous_path);
                }
                let kind = if change.is_directory {
                    SnapshotKind::Directory
                } else {
                    SnapshotKind::File
                };
                self.entries.insert(change.path.clone(), kind);
            }
        }
    }

    fn kind_for_existing_or_cached(&self, path: &Path) -> SnapshotKind {
        path.metadata()
            .map(|metadata| {
                if metadata.is_dir() {
                    SnapshotKind::Directory
                } else {
                    SnapshotKind::File
                }
            })
            .ok()
            .or_else(|| self.entries.get(path).copied())
            .unwrap_or(SnapshotKind::File)
    }
}

fn map_event(event: &Event, options: &WatchOptions, cache: &KindCache) -> Vec<AppEvent> {
    if event.need_rescan() {
        return Vec::new();
    }

    match event.kind {
        EventKind::Create(_) => map_simple_paths(event, options, cache, &ChangeKind::Created),
        EventKind::Remove(_) => map_simple_paths(event, options, cache, &ChangeKind::Deleted),
        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
            map_rename(event, options, cache).into_iter().collect()
        }
        EventKind::Modify(
            ModifyKind::Data(_) | ModifyKind::Metadata(_) | ModifyKind::Any | ModifyKind::Other,
        )
        | EventKind::Any => map_simple_paths(event, options, cache, &ChangeKind::Modified),
        EventKind::Modify(ModifyKind::Name(_)) | EventKind::Other | EventKind::Access(_) => {
            Vec::new()
        }
    }
}

fn map_simple_paths(
    event: &Event,
    options: &WatchOptions,
    cache: &KindCache,
    kind: &ChangeKind,
) -> Vec<AppEvent> {
    event
        .paths
        .iter()
        .map(|path| {
            let snapshot_kind = cache.kind_for_existing_or_cached(path);
            (path, snapshot_kind)
        })
        .filter(|(path, snapshot_kind)| should_emit_path(path, options, kind, *snapshot_kind))
        .map(|(path, snapshot_kind)| file_changed(path.clone(), kind.clone(), snapshot_kind))
        .collect()
}

fn map_rename(event: &Event, options: &WatchOptions, cache: &KindCache) -> Option<AppEvent> {
    let old_path = event.paths.first()?;
    let new_path = event.paths.get(1)?;
    let new_kind = cache.kind_for_existing_or_cached(new_path);
    if !should_emit_path(old_path, options, &ChangeKind::Renamed, new_kind)
        && !should_emit_path(new_path, options, &ChangeKind::Renamed, new_kind)
    {
        return None;
    }

    Some(file_renamed(old_path.clone(), new_path.clone(), new_kind))
}

fn should_emit_path(
    path: &Path,
    options: &WatchOptions,
    kind: &ChangeKind,
    snapshot_kind: SnapshotKind,
) -> bool {
    match kind {
        ChangeKind::Deleted => {
            !crate::filter::should_ignore(path, &options.ignore)
                && (snapshot_kind == SnapshotKind::Directory
                    || extension_allowed_for_deleted_file(path, options))
        }
        ChangeKind::Renamed => !crate::filter::should_ignore(path, &options.ignore),
        ChangeKind::Created | ChangeKind::Modified => path_allowed(path, options),
    }
}

fn extension_allowed_for_deleted_file(path: &Path, options: &WatchOptions) -> bool {
    options.extensions.is_empty()
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                options
                    .extensions
                    .iter()
                    .any(|allowed| allowed == extension)
            })
}

#[cfg(test)]
mod tests {
    use super::KindCache;
    use crate::snapshot::{FileSnapshot, SnapshotKind};
    use crate::watcher::WatchOptions;
    use notify::event::{ModifyKind, RenameMode};
    use notify::{Event, EventKind};
    use rumon_shared::{AppEvent, ChangeKind, WatchEvent};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn maps_rename_both_event() {
        let old_path = PathBuf::from("src/old.rs");
        let new_path = PathBuf::from("src/new.rs");
        let mut snapshots = BTreeMap::new();
        snapshots.insert(
            old_path.clone(),
            FileSnapshot {
                kind: SnapshotKind::File,
                len: 10,
                modified: Some(SystemTime::now()),
            },
        );
        let mut cache = KindCache::new(&snapshots);
        let event = Event::new(EventKind::Modify(ModifyKind::Name(RenameMode::Both)))
            .add_path(old_path)
            .add_path(new_path);
        let options = WatchOptions {
            paths: vec![PathBuf::from("src")],
            ignore: Vec::new(),
            extensions: vec!["rs".to_string()],
            recursive: true,
            follow_symlink: false,
        };

        let events = cache.map_event(&event, &options);

        assert!(events.iter().any(|event| matches!(
            event,
            AppEvent::Watch(WatchEvent::Changed(change))
                if change.kind == ChangeKind::Renamed
                    && change.previous_path.as_ref().is_some_and(|path| path.ends_with("old.rs"))
                    && change.path.ends_with("new.rs")
        )));
    }
}
