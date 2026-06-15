//! Diff output formatting.

use crate::binary::BinaryDiff;
use crate::line_diff::LineChange;

/// Formats line changes as unified-style preview lines.
#[must_use]
pub fn format_lines(changes: &[LineChange]) -> Vec<String> {
    changes.iter().map(LineChange::to_preview_line).collect()
}

/// Formats binary diff metadata for the UI.
#[must_use]
pub fn format_binary(diff: &BinaryDiff) -> Vec<String> {
    vec![
        "binary modified".to_string(),
        format!("size: {} -> {}", diff.before.size, diff.after.size),
        format!("hash changed: {}", diff.hash_changed),
    ]
}
