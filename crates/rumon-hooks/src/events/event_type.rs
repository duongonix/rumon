//! Supported event hook event types.

use rumon_shared::{ChangeDetail, ChangeKind, FileChange};

/// Event types available to rule-based event hooks.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EventType {
    /// File created.
    FileCreated,
    /// File modified.
    FileModified,
    /// File deleted.
    FileDeleted,
    /// File renamed.
    FileRenamed,
    /// Folder created.
    FolderCreated,
    /// Folder deleted.
    FolderDeleted,
    /// Folder renamed.
    FolderRenamed,
    /// File metadata changed.
    MetadataChanged,
    /// File content changed.
    ContentChanged,
    /// File permission changed.
    PermissionChanged,
}

impl EventType {
    /// Returns the config/env string for the event type.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FileCreated => "file_created",
            Self::FileModified => "file_modified",
            Self::FileDeleted => "file_deleted",
            Self::FileRenamed => "file_renamed",
            Self::FolderCreated => "folder_created",
            Self::FolderDeleted => "folder_deleted",
            Self::FolderRenamed => "folder_renamed",
            Self::MetadataChanged => "metadata_changed",
            Self::ContentChanged => "content_changed",
            Self::PermissionChanged => "permission_changed",
        }
    }

    /// Parses an event type string.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "file_created" => Some(Self::FileCreated),
            "file_modified" => Some(Self::FileModified),
            "file_deleted" => Some(Self::FileDeleted),
            "file_renamed" => Some(Self::FileRenamed),
            "folder_created" => Some(Self::FolderCreated),
            "folder_deleted" => Some(Self::FolderDeleted),
            "folder_renamed" => Some(Self::FolderRenamed),
            "metadata_changed" => Some(Self::MetadataChanged),
            "content_changed" => Some(Self::ContentChanged),
            "permission_changed" => Some(Self::PermissionChanged),
            _ => None,
        }
    }

    /// Infers the primary event type for a filesystem change.
    #[must_use]
    pub fn from_change(change: &FileChange) -> Self {
        match (&change.kind, change.is_directory) {
            (ChangeKind::Created, false) => Self::FileCreated,
            (ChangeKind::Created, true) => Self::FolderCreated,
            (ChangeKind::Modified, false) => Self::FileModified,
            (ChangeKind::Modified, true) => Self::MetadataChanged,
            (ChangeKind::Deleted, false) => Self::FileDeleted,
            (ChangeKind::Deleted, true) => Self::FolderDeleted,
            (ChangeKind::Renamed, false) => Self::FileRenamed,
            (ChangeKind::Renamed, true) => Self::FolderRenamed,
        }
    }

    /// Returns all event types that should match hook rules for a change.
    #[must_use]
    pub fn matching_types(change: &FileChange) -> Vec<Self> {
        let primary = Self::from_change(change);
        let mut events = vec![primary];
        if change.kind == ChangeKind::Modified && !change.is_directory {
            if matches!(change.detail, Some(ChangeDetail::Text { .. })) {
                events.push(Self::ContentChanged);
            } else if change.detail.is_some() {
                events.push(Self::MetadataChanged);
            }
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::EventType;

    #[test]
    fn parses_config_event_names() {
        assert_eq!(
            EventType::parse("file_modified"),
            Some(EventType::FileModified)
        );
        assert_eq!(EventType::parse("nope"), None);
    }
}
