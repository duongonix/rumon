//! Serializable protocol schema.

use serde::{Deserialize, Serialize};

/// File event kind exposed by Rumon integrations.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RumonEventType {
    /// A file was created.
    FileCreated,
    /// A file was modified.
    FileModified,
    /// A file was deleted.
    FileDeleted,
    /// A file was renamed.
    FileRenamed,
    /// A folder was created.
    FolderCreated,
    /// A folder was deleted.
    FolderDeleted,
    /// A folder was renamed.
    FolderRenamed,
    /// Metadata changed.
    MetadataChanged,
    /// Content changed.
    ContentChanged,
    /// Permissions changed.
    PermissionChanged,
}

/// Serializable Rumon event.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct RumonEvent {
    /// Stable event id for this process.
    pub id: String,
    /// Event type.
    #[serde(rename = "type")]
    pub event_type: RumonEventType,
    /// UTC timestamp in RFC3339-like form.
    pub timestamp: String,
    /// Active profile name or `none`.
    pub profile: String,
    /// Event path for create/modify/delete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Old path for rename events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    /// New path for rename events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_path: Option<String>,
    /// File metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FileInfo>,
    /// Diff summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<DiffInfo>,
    /// Metadata summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataInfo>,
    /// Media summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<MediaInfo>,
}

/// File metadata exposed to protocol clients.
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct FileInfo {
    /// File name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// File extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<String>,
    /// Whether the path is a directory.
    pub is_folder: bool,
}

/// Diff summary.
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[allow(clippy::struct_field_names)]
pub struct DiffInfo {
    /// Added line count.
    pub added_lines: usize,
    /// Removed line count.
    pub removed_lines: usize,
    /// Total changed line count.
    pub changed_lines: usize,
}

/// Metadata change summary.
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataInfo {
    /// Whether file size changed.
    pub size_changed: bool,
    /// Whether permissions changed.
    pub permissions_changed: bool,
}

/// Media metadata summary placeholder.
#[derive(Debug, Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MediaInfo {
    /// Media MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
    /// Media family, such as image/audio/video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// File size in bytes when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Runtime status response.
#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct StatusInfo {
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// Active profile name.
    pub profile: String,
    /// Whether command process is running.
    pub running: bool,
    /// Whether watcher is active.
    pub watching: bool,
    /// Watched paths.
    pub watch_paths: Vec<String>,
    /// Number of buffered events.
    pub event_count: usize,
    /// Server uptime in milliseconds.
    pub uptime_ms: u128,
}
