//! Diff preview model.

use crate::line_diff::LineChange;

/// A compact diff preview produced for display.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffPreview {
    /// Human-readable preview lines.
    pub lines: Vec<String>,
    /// Whether output was truncated by a preview limit.
    pub truncated: bool,
}

impl DiffPreview {
    /// Creates an empty diff preview.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            lines: Vec::new(),
            truncated: false,
        }
    }

    /// Builds a preview from line changes.
    #[must_use]
    pub fn from_changes(changes: &[LineChange], max_lines: usize) -> Self {
        let all_lines: Vec<String> = changes
            .iter()
            .flat_map(LineChange::to_preview_lines)
            .collect();
        let mut lines: Vec<String> = all_lines.iter().take(max_lines).cloned().collect();
        let truncated = all_lines.len() > max_lines;
        if truncated {
            lines.push(format!("... {} more changes", all_lines.len() - max_lines));
        }
        Self { lines, truncated }
    }
}

#[cfg(test)]
mod tests {
    use super::DiffPreview;
    use crate::{InlineDiff, LineChange};

    #[test]
    fn empty_preview_has_no_lines() {
        assert!(DiffPreview::empty().lines.is_empty());
    }

    #[test]
    fn modified_changes_do_not_embed_newlines() {
        let preview = DiffPreview::from_changes(
            &[LineChange::Modified {
                old_line: 1,
                new_line: 1,
                old_text: "}".to_string(),
                new_text: "} else {".to_string(),
                columns: None,
                inline: InlineDiff {
                    columns: None,
                    removed: String::new(),
                    added: String::new(),
                },
            }],
            4,
        );

        assert_eq!(preview.lines, vec!["- }", "+ } else {"]);
        assert!(!preview.lines.iter().any(|line| line.contains('\n')));
    }
}
