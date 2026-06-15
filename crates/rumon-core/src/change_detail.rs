//! File change detail enrichment.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rumon_config::WatchConfig;
use rumon_diff::{LineChange, diff_binary, diff_text, is_text_path, metadata};
use rumon_media::{MediaDetails, detect_media};
use rumon_shared::{AppEvent, ChangeDetail, ChangeKind, FileChange, WatchEvent};

/// Adds type-aware details to watch changes.
#[derive(Debug, Default)]
pub struct ChangeDetailInspector {
    snapshots: BTreeMap<PathBuf, Vec<u8>>,
}

impl ChangeDetailInspector {
    /// Seeds the previous-content cache from configured watch roots.
    pub fn seed_from_watch_config(&mut self, config: &WatchConfig) {
        for path in &config.paths {
            self.seed_path(path, config);
        }
    }

    /// Enriches an application event when it contains a file change.
    #[must_use]
    pub fn enrich_event(&mut self, event: AppEvent) -> AppEvent {
        let AppEvent::Watch(WatchEvent::Changed(change)) = event else {
            return event;
        };

        AppEvent::Watch(WatchEvent::Changed(self.enrich_change(change)))
    }

    fn enrich_change(&mut self, mut change: FileChange) -> FileChange {
        change.detail = match change.kind {
            ChangeKind::Deleted => {
                self.snapshots.remove(&change.path);
                Some(ChangeDetail::Deleted)
            }
            ChangeKind::Renamed => {
                if let Some(previous) = self.take_previous_snapshot(&change) {
                    self.detail_for_existing_path_with_previous(&change.path, Some(&previous))
                } else {
                    self.detail_for_existing_path(&change.path)
                }
            }
            ChangeKind::Created | ChangeKind::Modified => {
                self.detail_for_existing_path(&change.path)
            }
        };
        change
    }

    fn detail_for_existing_path(&mut self, path: &Path) -> Option<ChangeDetail> {
        let bytes = fs::read(path).ok()?;
        let previous = self.snapshots.insert(path.to_path_buf(), bytes.clone());
        Some(detail_from_bytes(path, previous.as_deref(), &bytes))
    }

    fn detail_for_existing_path_with_previous(
        &mut self,
        path: &Path,
        previous: Option<&[u8]>,
    ) -> Option<ChangeDetail> {
        let bytes = fs::read(path).ok()?;
        self.snapshots.insert(path.to_path_buf(), bytes.clone());
        Some(detail_from_bytes(path, previous, &bytes))
    }

    fn take_previous_snapshot(&mut self, change: &FileChange) -> Option<Vec<u8>> {
        change
            .previous_path
            .as_ref()
            .and_then(|path| self.snapshots.remove(path))
    }

    fn seed_path(&mut self, path: &Path, config: &WatchConfig) {
        if should_ignore(path, &config.ignore) {
            return;
        }

        let Ok(metadata) = fs::metadata(path) else {
            return;
        };

        if metadata.is_dir() {
            if !config.recursive && !config.paths.iter().any(|root| root == path) {
                return;
            }
            let Ok(entries) = fs::read_dir(path) else {
                return;
            };
            for entry in entries.flatten() {
                self.seed_path(&entry.path(), config);
            }
        } else if metadata.is_file()
            && extension_allowed(path, &config.extensions)
            && let Ok(bytes) = fs::read(path)
        {
            self.snapshots.insert(path.to_path_buf(), bytes);
        }
    }
}

fn should_ignore(path: &Path, ignore: &[PathBuf]) -> bool {
    ignore.iter().any(|ignored| {
        path == ignored
            || path.starts_with(ignored)
            || ignored.file_name().is_some_and(|name| {
                path.components()
                    .any(|component| component.as_os_str() == name)
            })
    })
}

fn extension_allowed(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.iter().any(|allowed| allowed == extension))
}

fn detail_from_bytes(path: &Path, previous: Option<&[u8]>, bytes: &[u8]) -> ChangeDetail {
    if is_text_path(path) {
        return text_detail(previous, bytes);
    }

    if let Some(media) = detect_media(path, bytes) {
        return media_detail(media);
    }

    binary_detail(previous, bytes)
}

fn text_detail(previous: Option<&[u8]>, current: &[u8]) -> ChangeDetail {
    let current_text = String::from_utf8_lossy(current);
    let previous_text = previous.map(String::from_utf8_lossy);
    let diff = diff_text(
        previous_text.as_deref().unwrap_or_default(),
        &current_text,
        4,
    );
    let location = diff.changes.iter().find_map(location_summary);
    let preview = if diff.preview.lines.is_empty() {
        current_text
            .lines()
            .take(4)
            .map(|line| format!("+ {line}"))
            .collect()
    } else {
        diff.preview.lines
    };

    ChangeDetail::Text {
        location,
        preview,
        truncated: diff.preview.truncated,
    }
}

fn location_summary(change: &LineChange) -> Option<String> {
    match change {
        LineChange::Added { new_line, .. } => Some(format!("line {new_line}")),
        LineChange::Removed { old_line, .. } => Some(format!("line {old_line}")),
        LineChange::Modified {
            new_line, columns, ..
        } => columns.map_or_else(
            || Some(format!("line {new_line}")),
            |columns| Some(format!("line {new_line} col {}", columns.start)),
        ),
        LineChange::Unchanged { .. } => None,
    }
}

fn media_detail(media: rumon_media::MediaSummary) -> ChangeDetail {
    let mut metadata = Vec::new();
    match &media.details {
        MediaDetails::Image(image) => {
            if let (Some(width), Some(height)) = (image.width, image.height) {
                metadata.push(format!("{width}x{height}"));
            }
            if let Some(color_type) = &image.color_type {
                metadata.push(color_type.clone());
            }
        }
        MediaDetails::Audio(audio) => {
            push_optional(
                &mut metadata,
                "duration",
                audio.duration_ms.map(format_duration),
            );
            push_optional(
                &mut metadata,
                "bitrate",
                audio.bitrate.map(|value| format!("{value} bps")),
            );
            push_optional(
                &mut metadata,
                "channels",
                audio.channels.map(|value| value.to_string()),
            );
            push_optional(
                &mut metadata,
                "sample rate",
                audio.sample_rate.map(|value| format!("{value} Hz")),
            );
        }
        MediaDetails::Video(video) => {
            if let (Some(width), Some(height)) = (video.width, video.height) {
                metadata.push(format!("{width}x{height}"));
            }
            push_optional(
                &mut metadata,
                "duration",
                video.duration_ms.map(format_duration),
            );
            if let Some(codec) = &video.codec {
                metadata.push(format!("codec {codec}"));
            }
            if let Some(frame_rate) = video.frame_rate {
                metadata.push(format!("frame rate {frame_rate:.2}"));
            }
        }
    }

    ChangeDetail::Media {
        kind: media.kind.to_string(),
        mime_type: media.mime_type,
        size_bytes: media.file_size,
        metadata,
    }
}

fn binary_detail(previous: Option<&[u8]>, current: &[u8]) -> ChangeDetail {
    let current_metadata = metadata(current);
    let (previous_size, hash_changed) = previous.map_or((None, false), |previous| {
        let diff = diff_binary(previous, current);
        (Some(diff.before.size), diff.hash_changed)
    });

    ChangeDetail::Binary {
        previous_size,
        current_size: Some(current_metadata.size),
        hash_changed,
    }
}

fn push_optional(lines: &mut Vec<String>, label: &str, value: Option<String>) {
    if let Some(value) = value {
        lines.push(format!("{label} {value}"));
    }
}

fn format_duration(milliseconds: u64) -> String {
    let total_seconds = milliseconds / 1_000;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

#[cfg(test)]
mod tests {
    use super::ChangeDetailInspector;
    use rumon_shared::{AppEvent, ChangeDetail, ChangeKind, FileChange, WatchEvent};
    use std::fs;

    #[test]
    fn enriches_text_changes() {
        let path = std::env::temp_dir().join("rumon_text_detail_test.rs");
        fs::write(&path, "let port = 3000;").expect("write test file");
        let mut inspector = ChangeDetailInspector::default();

        let event = inspector.enrich_event(AppEvent::Watch(WatchEvent::Changed(FileChange {
            path: path.clone(),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        })));

        let _ = fs::remove_file(path);

        let AppEvent::Watch(WatchEvent::Changed(change)) = event else {
            panic!("expected watch change");
        };
        assert!(matches!(change.detail, Some(ChangeDetail::Text { .. })));
    }
}
