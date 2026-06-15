//! Column and inline change detection.

/// A changed column range within a modified line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColumnChange {
    /// First changed column, 1-based.
    pub start: usize,
    /// Last changed column, 1-based and inclusive.
    pub end: usize,
}

/// Inline diff fragments for a modified line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InlineDiff {
    /// Removed fragment.
    pub removed: String,
    /// Added fragment.
    pub added: String,
    /// Changed column range.
    pub columns: Option<ColumnChange>,
}

/// Computes an inline diff by trimming common prefix and suffix.
#[must_use]
pub fn inline_diff(before: &str, after: &str) -> InlineDiff {
    let prefix = common_prefix_len(before, after);
    let before_tail = &before[prefix..];
    let after_tail = &after[prefix..];
    let suffix = common_suffix_len(before_tail, after_tail);

    let before_changed_end = before.len().saturating_sub(suffix);
    let after_changed_end = after.len().saturating_sub(suffix);
    let removed = before[prefix..before_changed_end].to_string();
    let added = after[prefix..after_changed_end].to_string();
    let changed_width = removed.chars().count().max(added.chars().count());
    let columns = (changed_width > 0).then_some(ColumnChange {
        start: before[..prefix].chars().count() + 1,
        end: before[..prefix].chars().count() + changed_width,
    });

    InlineDiff {
        removed,
        added,
        columns,
    }
}

fn common_prefix_len(before: &str, after: &str) -> usize {
    before
        .char_indices()
        .zip(after.char_indices())
        .take_while(|((_, left), (_, right))| left == right)
        .map(|((index, ch), _)| index + ch.len_utf8())
        .last()
        .unwrap_or(0)
}

fn common_suffix_len(before: &str, after: &str) -> usize {
    before
        .chars()
        .rev()
        .zip(after.chars().rev())
        .take_while(|(left, right)| left == right)
        .map(|(ch, _)| ch.len_utf8())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::{ColumnChange, inline_diff};

    #[test]
    fn detects_changed_columns() {
        let diff = inline_diff("let port = 3000;", "let port = 8080;");

        assert_eq!(diff.removed, "300");
        assert_eq!(diff.added, "808");
        assert_eq!(diff.columns, Some(ColumnChange { start: 12, end: 14 }));
    }
}
