//! Filesystem snapshot models and scanning.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ignore::{DirEntry, WalkBuilder};

use crate::filter::{extension_allowed, should_ignore};
use crate::watcher::WatchOptions;

/// A lightweight filesystem snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileSnapshot {
    /// Snapshot entry kind.
    pub kind: SnapshotKind,
    /// File size in bytes.
    pub len: u64,
    /// Last modified time when available.
    pub modified: Option<SystemTime>,
}

/// Filesystem entry kind stored in a snapshot.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SnapshotKind {
    /// Regular file.
    File,
    /// Directory.
    Directory,
}

impl FileSnapshot {
    /// Returns whether two snapshots can be treated as the same entry after a rename.
    #[must_use]
    pub fn rename_matches(&self, other: &Self) -> bool {
        match (self.kind, other.kind) {
            (SnapshotKind::File, SnapshotKind::File) => self == other,
            (SnapshotKind::Directory, SnapshotKind::Directory) => true,
            _ => false,
        }
    }
}

/// Scans all configured watch roots.
pub(crate) fn scan(options: &WatchOptions) -> std::io::Result<BTreeMap<PathBuf, FileSnapshot>> {
    let mut snapshots = BTreeMap::new();
    let mut paths = options.paths.iter();
    let Some(first_path) = paths.next() else {
        return Ok(snapshots);
    };

    let mut builder = WalkBuilder::new(first_path);
    for path in paths {
        builder.add(path);
    }
    builder
        .follow_links(options.follow_symlink)
        .standard_filters(true)
        .hidden(false)
        .max_depth((!options.recursive).then_some(1));

    let ignore_paths = options.ignore.clone();
    let extensions = options.extensions.clone();
    builder.filter_entry(move |entry| entry_allowed(entry, &ignore_paths, &extensions));

    for result in builder.build() {
        let entry = result.map_err(|error| std::io::Error::other(error.to_string()))?;
        scan_entry(entry.path(), &mut snapshots)?;
    }
    Ok(snapshots)
}

fn scan_entry(path: &Path, snapshots: &mut BTreeMap<PathBuf, FileSnapshot>) -> std::io::Result<()> {
    let metadata = path.metadata()?;
    if metadata.is_dir() {
        snapshots.insert(
            path.to_path_buf(),
            FileSnapshot {
                kind: SnapshotKind::Directory,
                len: 0,
                modified: metadata.modified().ok(),
            },
        );
    } else if metadata.is_file() {
        snapshots.insert(
            path.to_path_buf(),
            FileSnapshot {
                kind: SnapshotKind::File,
                len: metadata.len(),
                modified: metadata.modified().ok(),
            },
        );
    }

    Ok(())
}

fn entry_allowed(entry: &DirEntry, ignore_paths: &[PathBuf], extensions: &[String]) -> bool {
    let path = entry.path();
    if should_ignore(path, ignore_paths) {
        return false;
    }
    entry
        .file_type()
        .is_none_or(|file_type| file_type.is_dir() || extension_allowed(path, extensions))
}
