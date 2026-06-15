//! Line-based text diffing.

use crate::column_diff::{ColumnChange, InlineDiff, inline_diff};
use similar::{ChangeTag, TextDiff};

/// A line change produced by the diff engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LineChange {
    /// Unchanged line.
    Unchanged {
        /// 1-based line number in the old file.
        old_line: usize,
        /// 1-based line number in the new file.
        new_line: usize,
        /// Line text.
        text: String,
    },
    /// Added line.
    Added {
        /// 1-based line number in the new file.
        new_line: usize,
        /// Line text.
        text: String,
    },
    /// Removed line.
    Removed {
        /// 1-based line number in the old file.
        old_line: usize,
        /// Line text.
        text: String,
    },
    /// Modified line.
    Modified {
        /// 1-based line number in the old file.
        old_line: usize,
        /// 1-based line number in the new file.
        new_line: usize,
        /// Previous text.
        old_text: String,
        /// Current text.
        new_text: String,
        /// Optional changed column range.
        columns: Option<ColumnChange>,
        /// Inline diff fragments.
        inline: InlineDiff,
    },
}

impl LineChange {
    /// Formats this line change as a diff preview line.
    #[must_use]
    pub fn to_preview_line(&self) -> String {
        match self {
            Self::Unchanged { text, .. } => format!("  {text}"),
            Self::Added { text, .. } => format!("+ {text}"),
            Self::Removed { text, .. } => format!("- {text}"),
            Self::Modified {
                old_text, new_text, ..
            } => format!("- {old_text}\n+ {new_text}"),
        }
    }

    /// Formats this line change as one or more diff preview rows.
    #[must_use]
    pub fn to_preview_lines(&self) -> Vec<String> {
        match self {
            Self::Modified {
                old_text, new_text, ..
            } => vec![format!("- {old_text}"), format!("+ {new_text}")],
            _ => vec![self.to_preview_line()],
        }
    }

    /// Returns whether this line is changed.
    #[must_use]
    pub const fn is_changed(&self) -> bool {
        !matches!(self, Self::Unchanged { .. })
    }
}

/// Computes line changes between two text documents.
#[must_use]
pub fn diff_lines(before: &str, after: &str) -> Vec<LineChange> {
    let diff = TextDiff::from_lines(before, after);
    let mut changes = Vec::new();
    for op in diff.ops() {
        let raw = diff
            .iter_changes(op)
            .map(|change| {
                let text = trim_line_ending(change.value()).to_string();
                match change.tag() {
                    ChangeTag::Equal => LineChange::Unchanged {
                        old_line: change.old_index().map_or(0, |index| index + 1),
                        new_line: change.new_index().map_or(0, |index| index + 1),
                        text,
                    },
                    ChangeTag::Delete => LineChange::Removed {
                        old_line: change.old_index().map_or(0, |index| index + 1),
                        text,
                    },
                    ChangeTag::Insert => LineChange::Added {
                        new_line: change.new_index().map_or(0, |index| index + 1),
                        text,
                    },
                }
            })
            .collect::<Vec<_>>();
        changes.extend(combine_modifications(&raw));
    }
    changes
}

fn trim_line_ending(value: &str) -> &str {
    value.trim_end_matches(['\r', '\n'])
}

fn combine_modifications(changes: &[LineChange]) -> Vec<LineChange> {
    let removed = removed_lines(changes);
    let added = added_lines(changes);
    if removed.is_empty() || added.is_empty() {
        return changes.to_vec();
    }

    let mut combined = Vec::new();
    let mut removed_start = 0;
    let mut added_start = 0;
    while removed_start < removed.len()
        && added_start < added.len()
        && removed[removed_start].1 == added[added_start].1
    {
        let old = &removed[removed_start];
        let new = &added[added_start];
        combined.push(LineChange::Unchanged {
            old_line: old.0,
            new_line: new.0,
            text: old.1.clone(),
        });
        removed_start += 1;
        added_start += 1;
    }

    let mut removed_end = removed.len();
    let mut added_end = added.len();
    let mut suffix = Vec::new();
    while removed_end > removed_start
        && added_end > added_start
        && removed[removed_end - 1].1 == added[added_end - 1].1
    {
        removed_end -= 1;
        added_end -= 1;
        let old = &removed[removed_end];
        let new = &added[added_end];
        suffix.push(LineChange::Unchanged {
            old_line: old.0,
            new_line: new.0,
            text: old.1.clone(),
        });
    }

    let remaining_removed = &removed[removed_start..removed_end];
    let remaining_added = &added[added_start..added_end];
    let pair_count = remaining_removed.len().min(remaining_added.len());
    for (old, new) in removed[removed_start..removed_end]
        .iter()
        .take(pair_count)
        .zip(added[added_start..added_end].iter().take(pair_count))
    {
        combined.push(modified_line(old.0, &old.1, new.0, &new.1));
    }
    combined.extend(
        remaining_removed
            .iter()
            .skip(pair_count)
            .map(|(old_line, text)| LineChange::Removed {
                old_line: *old_line,
                text: text.clone(),
            }),
    );
    combined.extend(
        remaining_added
            .iter()
            .skip(pair_count)
            .map(|(new_line, text)| LineChange::Added {
                new_line: *new_line,
                text: text.clone(),
            }),
    );
    suffix.reverse();
    combined.extend(suffix);
    combined
}

fn removed_lines(changes: &[LineChange]) -> Vec<(usize, String)> {
    changes
        .iter()
        .filter_map(|change| match change {
            LineChange::Removed { old_line, text } => Some((*old_line, text.clone())),
            _ => None,
        })
        .collect()
}

fn added_lines(changes: &[LineChange]) -> Vec<(usize, String)> {
    changes
        .iter()
        .filter_map(|change| match change {
            LineChange::Added { new_line, text } => Some((*new_line, text.clone())),
            _ => None,
        })
        .collect()
}

fn modified_line(old_line: usize, old_text: &str, new_line: usize, new_text: &str) -> LineChange {
    let inline = inline_diff(old_text, new_text);
    LineChange::Modified {
        old_line,
        new_line,
        old_text: old_text.to_string(),
        new_text: new_text.to_string(),
        columns: inline.columns,
        inline,
    }
}

#[cfg(test)]
mod tests {
    use super::{LineChange, diff_lines};
    use crate::ColumnChange;

    #[test]
    fn detects_added_removed_and_modified_lines() {
        let changes = diff_lines("a\nold\nz", "a\nnew\nz\nextra");

        assert!(matches!(changes[0], LineChange::Unchanged { .. }));
        assert!(matches!(
            changes[1],
            LineChange::Modified {
                columns: Some(ColumnChange { .. }),
                ..
            }
        ));
        assert!(matches!(changes[3], LineChange::Added { .. }));
    }

    #[test]
    fn modified_preview_lines_are_split() {
        let change = LineChange::Modified {
            old_line: 1,
            new_line: 1,
            old_text: "}".to_string(),
            new_text: "} else {".to_string(),
            columns: None,
            inline: crate::InlineDiff {
                columns: None,
                removed: String::new(),
                added: String::new(),
            },
        };

        assert_eq!(
            change.to_preview_lines(),
            vec!["- }".to_string(), "+ } else {".to_string()]
        );
    }
}
