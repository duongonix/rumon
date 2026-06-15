//! Conversion from internal app events to protocol events.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rumon_shared::{AppEvent, ChangeDetail, ChangeKind, FileChange, WatchEvent, display_path};

use crate::schema::{DiffInfo, FileInfo, MediaInfo, MetadataInfo, RumonEvent, RumonEventType};

static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

/// Converts an application event into a serializable Rumon event when possible.
#[must_use]
pub fn app_event_to_rumon_event(event: &AppEvent, profile: Option<&str>) -> Option<RumonEvent> {
    let AppEvent::Watch(WatchEvent::Changed(change)) = event else {
        return None;
    };
    Some(change_to_event(change, profile.unwrap_or("none")))
}

/// Maps a file change to a protocol event type.
#[must_use]
pub fn event_type_for_change(change: &FileChange) -> RumonEventType {
    match (&change.kind, change.is_directory) {
        (ChangeKind::Created, true) => RumonEventType::FolderCreated,
        (ChangeKind::Created, false) => RumonEventType::FileCreated,
        (ChangeKind::Modified, _) => RumonEventType::FileModified,
        (ChangeKind::Deleted, true) => RumonEventType::FolderDeleted,
        (ChangeKind::Deleted, false) => RumonEventType::FileDeleted,
        (ChangeKind::Renamed, true) => RumonEventType::FolderRenamed,
        (ChangeKind::Renamed, false) => RumonEventType::FileRenamed,
    }
}

fn change_to_event(change: &FileChange, profile: &str) -> RumonEvent {
    let is_rename = change.kind == ChangeKind::Renamed;
    RumonEvent {
        id: next_id(),
        event_type: event_type_for_change(change),
        timestamp: timestamp(),
        profile: profile.to_string(),
        path: (!is_rename).then(|| display_path(&change.path)),
        old_path: change.previous_path.as_ref().map(|path| display_path(path)),
        new_path: is_rename.then(|| display_path(&change.path)),
        file: Some(file_info(change)),
        diff: diff_info(change),
        metadata: metadata_info(change),
        media: media_info(change),
    }
}

fn file_info(change: &FileChange) -> FileInfo {
    FileInfo {
        name: change
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned),
        ext: change
            .path
            .extension()
            .and_then(|value| value.to_str())
            .map(ToOwned::to_owned),
        is_folder: change.is_directory,
    }
}

fn diff_info(change: &FileChange) -> Option<DiffInfo> {
    let Some(ChangeDetail::Text { preview, .. }) = &change.detail else {
        return None;
    };
    let added_lines = preview.iter().filter(|line| line.starts_with('+')).count();
    let removed_lines = preview.iter().filter(|line| line.starts_with('-')).count();
    Some(DiffInfo {
        added_lines,
        removed_lines,
        changed_lines: added_lines.saturating_add(removed_lines),
    })
}

fn metadata_info(change: &FileChange) -> Option<MetadataInfo> {
    matches!(change.detail, Some(ChangeDetail::Binary { .. })).then_some(MetadataInfo {
        size_changed: true,
        permissions_changed: false,
    })
}

fn media_info(change: &FileChange) -> Option<MediaInfo> {
    let Some(ChangeDetail::Media {
        kind,
        mime_type,
        size_bytes,
        ..
    }) = &change.detail
    else {
        return None;
    };
    Some(MediaInfo {
        mime: Some(mime_type.clone()),
        kind: Some(kind.clone()),
        size_bytes: Some(*size_bytes),
    })
}

fn next_id() -> String {
    let id = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);
    format!("evt_{id:06}")
}

fn timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    unix_millis_to_rfc3339(duration.as_secs(), duration.subsec_millis())
}

fn unix_millis_to_rfc3339(seconds: u64, millis: u32) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(i64::MAX);
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    if millis == 0 {
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    } else {
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
    }
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let days = days_since_epoch + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (
        i32::try_from(year).unwrap_or(i32::MAX),
        u32::try_from(month).unwrap_or(1),
        u32::try_from(day).unwrap_or(1),
    )
}

#[cfg(test)]
mod tests {
    use rumon_shared::{AppEvent, ChangeKind, FileChange, WatchEvent};

    use super::{app_event_to_rumon_event, unix_millis_to_rfc3339};
    use crate::RumonEventType;

    #[test]
    fn converts_file_change_event() {
        let event = AppEvent::Watch(WatchEvent::Changed(FileChange {
            path: "src/a.rs".into(),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        }));

        let protocol = app_event_to_rumon_event(&event, Some("rust")).expect("protocol event");

        assert_eq!(protocol.event_type, RumonEventType::FileModified);
        assert_eq!(protocol.path.as_deref(), Some("src/a.rs"));
        assert_eq!(protocol.profile, "rust");
    }

    #[test]
    fn formats_timestamp_as_rfc3339_utc() {
        assert_eq!(
            unix_millis_to_rfc3339(1_718_303_400, 0),
            "2024-06-13T18:30:00Z"
        );
        assert_eq!(
            unix_millis_to_rfc3339(1_718_303_400, 25),
            "2024-06-13T18:30:00.025Z"
        );
    }
}
