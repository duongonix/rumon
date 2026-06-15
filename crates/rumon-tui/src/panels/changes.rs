//! Changes panel rendering.

use std::collections::BTreeSet;
use std::path::Path;

use rumon_shared::{ChangeDetail, ChangeKind, FileChange, display_path};

use crate::theme::{ColorToken, bold_paint, bold_paint_on, dim, paint, paint_on};

/// Renders changed files as Git-style change blocks.
#[must_use]
pub fn render_changes_panel(
    changes: &[FileChange],
    selected: usize,
    scroll: usize,
    collapsed: &BTreeSet<usize>,
    width: usize,
    height: usize,
) -> Vec<String> {
    let mut lines = vec![title_line(changes.len())];
    if changes.is_empty() {
        lines.push(String::new());
        lines.push(format!("  {}", dim("No file changes yet")));
        lines.push(format!("  {}", dim("Waiting for filesystem events...")));
        return fit_height(lines, height);
    }

    let content_height = height.saturating_sub(lines.len());
    let mut content = Vec::new();
    for (index, change) in changes.iter().enumerate() {
        content.extend(format_change_block(
            change,
            index == selected,
            collapsed.contains(&index),
            width,
        ));
    }
    let max_scroll = content.len().saturating_sub(content_height);
    let scroll = scroll.min(max_scroll);
    let total_rows = content.len();
    let mut visible_content: Vec<String> = content
        .into_iter()
        .skip(scroll)
        .take(content_height)
        .collect();
    apply_scroll_indicator(&mut visible_content, scroll, total_rows, content_height);
    lines.extend(visible_content);
    fit_height(lines, height)
}

fn title_line(count: usize) -> String {
    format!(
        "{} {}",
        bold_paint(ColorToken::Info, "Changes"),
        dim(format!("({count} files)"))
    )
}

fn format_change_block(
    change: &FileChange,
    selected: bool,
    collapsed: bool,
    width: usize,
) -> Vec<String> {
    let inner = block_width(width);
    let token = change_token(change);
    let badge = status_badge(change, token);
    let path = header_path(change);
    let kind = kind_label(change);
    let line_counts = stats_label(change);
    let chevron = if collapsed { "▸" } else { "▾" };
    let path_width = inner.saturating_sub(9);
    let path = fit_visible(&paint(ColorToken::Text, path), path_width);
    let header = format!(" {badge}  {path}");
    let summary = format!(
        "{} {} {}",
        paint(ColorToken::Warning, kind),
        line_counts,
        paint(ColorToken::Text, chevron)
    );
    let status_line = format!("      {summary}");
    let border_token = if selected {
        ColorToken::BorderActive
    } else {
        ColorToken::Border
    };

    let mut lines = Vec::new();
    lines.push(paint(
        border_token,
        format!("┌{}┐", "─".repeat(inner.saturating_sub(2))),
    ));
    lines.push(paint(
        border_token,
        format!("│{}│", fit_visible(&header, inner.saturating_sub(2))),
    ));
    lines.push(paint(
        border_token,
        format!("│{}│", fit_visible(&status_line, inner.saturating_sub(2))),
    ));
    lines.push(paint(
        border_token,
        format!("├{}┤", "─".repeat(inner.saturating_sub(2))),
    ));
    if !collapsed {
        lines.extend(detail_lines(change, inner.saturating_sub(2)));
    }
    lines.push(paint(
        border_token,
        format!("└{}┘", "─".repeat(inner.saturating_sub(2))),
    ));
    lines.push(String::new());
    lines
}

fn block_width(width: usize) -> usize {
    width.saturating_mul(90).saturating_div(100).max(20)
}

fn status_badge(change: &FileChange, token: ColorToken) -> String {
    let label = match change.kind {
        ChangeKind::Created if is_folder_like(change) => "F",
        ChangeKind::Created => "A",
        ChangeKind::Modified => "M",
        ChangeKind::Deleted => "D",
        ChangeKind::Renamed => "R",
    };
    bold_paint_on(token, ColorToken::Surface, format!("[{label}]"))
}

fn change_token(change: &FileChange) -> ColorToken {
    match change.kind {
        ChangeKind::Created if is_folder_like(change) => ColorToken::Warning,
        ChangeKind::Created => ColorToken::Added,
        ChangeKind::Modified => ColorToken::Info,
        ChangeKind::Deleted => ColorToken::Removed,
        ChangeKind::Renamed => ColorToken::Warning,
    }
}

fn header_path(change: &FileChange) -> String {
    let path = if change.kind == ChangeKind::Renamed {
        change.previous_path.as_deref().unwrap_or(&change.path)
    } else {
        change.path.as_path()
    };
    normalize_path(path)
}

fn kind_label(change: &FileChange) -> &'static str {
    match change.kind {
        ChangeKind::Created => "created",
        ChangeKind::Modified => "modified",
        ChangeKind::Deleted => "deleted",
        ChangeKind::Renamed => "renamed",
    }
}

fn stats_label(change: &FileChange) -> String {
    let (added, removed) = diff_stats(change);
    match change.kind {
        ChangeKind::Created if added > 0 => paint(ColorToken::Added, format!("+{added}")),
        ChangeKind::Deleted if removed > 0 => paint(ColorToken::Removed, format!("-{removed}")),
        ChangeKind::Modified if added > 0 || removed > 0 => format!(
            "{} {} {}",
            paint(ColorToken::Added, format!("+{added}")),
            dim("/"),
            paint(ColorToken::Removed, format!("-{removed}"))
        ),
        _ => String::new(),
    }
}

fn diff_stats(change: &FileChange) -> (usize, usize) {
    let Some(ChangeDetail::Text { preview, .. }) = &change.detail else {
        return (0, 0);
    };
    let added = preview
        .iter()
        .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
        .count();
    let removed = preview
        .iter()
        .filter(|line| line.starts_with('-') && !line.starts_with("---"))
        .count();
    (added, removed)
}

fn detail_lines(change: &FileChange, width: usize) -> Vec<String> {
    match change.kind {
        ChangeKind::Deleted => {
            return vec![block_line(
                &paint(ColorToken::Removed, "file deleted"),
                width,
            )];
        }
        ChangeKind::Renamed => {
            return vec![block_line(&rename_line(change), width)];
        }
        ChangeKind::Created if is_folder_like(change) => {
            return vec![block_line(
                &paint(ColorToken::Warning, "folder created"),
                width,
            )];
        }
        _ => {}
    }

    match &change.detail {
        Some(ChangeDetail::Text {
            location,
            preview,
            truncated,
        }) => text_detail_lines(location.as_deref(), preview, *truncated, width),
        Some(ChangeDetail::Media {
            kind,
            mime_type,
            size_bytes,
            metadata,
        }) => media_detail_lines(kind, mime_type, *size_bytes, metadata, width),
        Some(ChangeDetail::Binary {
            previous_size,
            current_size,
            hash_changed,
        }) => binary_detail_lines(*previous_size, *current_size, *hash_changed, width),
        Some(ChangeDetail::Deleted) => vec![block_line(
            &paint(ColorToken::Removed, "file deleted"),
            width,
        )],
        None => vec![block_line(&dim("metadata changed"), width)],
    }
}

fn text_detail_lines(
    location: Option<&str>,
    preview: &[String],
    truncated: bool,
    width: usize,
) -> Vec<String> {
    let start_line = location.and_then(first_number).unwrap_or(1);
    let mut lines: Vec<String> = preview
        .iter()
        .flat_map(|line| line.lines())
        .take(7)
        .enumerate()
        .map(|(offset, line)| format_diff_line(start_line + offset, line, width))
        .collect();
    if truncated {
        lines.push(block_line(&dim("diff preview truncated"), width));
    }
    lines
}

fn format_diff_line(line_number: usize, line: &str, width: usize) -> String {
    let (prefix, body, token, background) = match line.chars().next() {
        Some('+') => (
            "+",
            &line[1..],
            ColorToken::Added,
            Some(ColorToken::AddedBackground),
        ),
        Some('-') => (
            "-",
            &line[1..],
            ColorToken::Removed,
            Some(ColorToken::RemovedBackground),
        ),
        _ => (" ", line, ColorToken::Text, None),
    };
    let gutter = format!(
        "{} {}{}",
        dim(format!("{line_number:>4}")),
        paint(token, prefix),
        dim("|")
    );
    let body = format!(" {}", body.trim_start());
    let row = match background {
        Some(background) => format!(
            "{gutter}{}",
            paint_on(
                token,
                background,
                fit_visible(&body, width.saturating_sub(8))
            )
        ),
        None => format!("{gutter}{}", paint(ColorToken::Text, body)),
    };
    block_line(&row, width)
}

fn media_detail_lines(
    kind: &str,
    mime_type: &str,
    size_bytes: u64,
    metadata: &[String],
    width: usize,
) -> Vec<String> {
    let mut lines = vec![
        block_line(&paint(ColorToken::Info, kind), width),
        block_line(&paint(ColorToken::Info, mime_type), width),
        block_line(&paint(ColorToken::Warning, format_size(size_bytes)), width),
    ];
    lines.extend(
        metadata
            .iter()
            .take(4)
            .map(|line| block_line(&paint(ColorToken::Text, line), width)),
    );
    lines
}

fn binary_detail_lines(
    previous_size: Option<u64>,
    current_size: Option<u64>,
    hash_changed: bool,
    width: usize,
) -> Vec<String> {
    let mut lines = vec![block_line(
        &paint(ColorToken::Muted, "metadata changed"),
        width,
    )];
    match (previous_size, current_size) {
        (Some(previous), Some(current)) if previous != current => lines.push(block_line(
            &format!(
                "{} {} {}",
                dim("size:"),
                paint(ColorToken::Warning, format_size(previous)),
                paint(ColorToken::Warning, format!("-> {}", format_size(current)))
            ),
            width,
        )),
        (_, Some(current)) => {
            lines.push(block_line(
                &paint(ColorToken::Warning, format_size(current)),
                width,
            ));
        }
        _ => {}
    }
    if hash_changed {
        lines.push(block_line(
            &paint(ColorToken::Changed, "hash changed"),
            width,
        ));
    }
    lines
}

fn rename_line(change: &FileChange) -> String {
    let previous = change
        .previous_path
        .as_deref()
        .map_or_else(|| normalize_path(&change.path), normalize_path);
    format!(
        "{} {} {} {}",
        dim("from"),
        paint(ColorToken::Warning, previous),
        dim("to"),
        paint(ColorToken::Warning, normalize_path(&change.path))
    )
}

fn block_line(value: &str, width: usize) -> String {
    format!("│{}│", fit_visible(&format!("  {value}"), width))
}

fn apply_scroll_indicator(lines: &mut [String], scroll: usize, total: usize, visible: usize) {
    if lines.is_empty() || total <= visible {
        return;
    }
    if scroll > 0 {
        let first = lines[0].clone();
        lines[0] = overlay_last_visible(&first, &paint(ColorToken::Info, "↑"));
    }
    if scroll.saturating_add(visible) < total {
        let last_index = lines.len().saturating_sub(1);
        let last = lines[last_index].clone();
        lines[last_index] = overlay_last_visible(&last, &paint(ColorToken::Info, "↓"));
    }
}

fn overlay_last_visible(line: &str, marker: &str) -> String {
    let mut visible = 0;
    let mut output = String::new();
    let mut chars = line.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            output.push(character);
            consume_ansi_sequence(&mut chars, &mut output);
            continue;
        }
        visible += 1;
        if visible + 1 >= visible_width(line) {
            output.push_str(marker);
        } else {
            output.push(character);
        }
    }
    output
}

fn is_folder_like(change: &FileChange) -> bool {
    change.is_directory
}

fn normalize_path(path: &Path) -> String {
    display_path(path)
}

fn first_number(value: &str) -> Option<usize> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|part| !part.is_empty())
        .and_then(|part| part.parse().ok())
}

fn format_size(bytes: u64) -> String {
    const KIB: u64 = 1_024;
    const MIB: u64 = KIB * 1_024;
    if bytes >= MIB {
        format_decimal_size(bytes, MIB, "MB")
    } else if bytes >= KIB {
        format_decimal_size(bytes, KIB, "KB")
    } else {
        format!("{bytes} B")
    }
}

fn format_decimal_size(bytes: u64, unit: u64, label: &str) -> String {
    let whole = bytes / unit;
    let tenth = (bytes % unit) * 10 / unit;
    format!("{whole}.{tenth} {label}")
}

fn fit_height(mut lines: Vec<String>, height: usize) -> Vec<String> {
    lines.truncate(height);
    while lines.len() < height {
        lines.push(String::new());
    }
    lines
}

fn fit_visible(value: &str, width: usize) -> String {
    let mut clipped = String::new();
    let mut current = 0;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            clipped.push(character);
            consume_ansi_sequence(&mut chars, &mut clipped);
            continue;
        }
        if current >= width {
            continue;
        }
        clipped.push(character);
        current += 1;
    }
    if current < width {
        clipped.push_str(&" ".repeat(width - current));
    }
    clipped
}

fn visible_width(value: &str) -> usize {
    let mut width = 0;
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            consume_escape_sequence(&mut chars);
        } else {
            width += 1;
        }
    }
    width
}

fn consume_ansi_sequence(
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
    output: &mut String,
) {
    if let Some(character) = chars.next() {
        output.push(character);
        if character != '[' {
            return;
        }
    }

    for character in chars.by_ref() {
        output.push(character);
        if ('@'..='~').contains(&character) {
            break;
        }
    }
}

fn consume_escape_sequence(chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) {
    if chars.next_if_eq(&'[').is_none() {
        return;
    }

    for character in chars.by_ref() {
        if ('@'..='~').contains(&character) {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::render_changes_panel;
    use rumon_shared::{ChangeDetail, ChangeKind, FileChange};

    #[test]
    fn renders_text_change_detail_without_hunk_header() {
        let changes = vec![FileChange {
            path: PathBuf::from("src/main.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: Some("line 15 col 10".to_string()),
                preview: vec![
                    "- let port = 3000;".to_string(),
                    "+ let port = 8080;".to_string(),
                ],
                truncated: false,
            }),
        }];

        let rendered = render_changes_panel(&changes, 0, 0, &BTreeSet::new(), 80, 12).join("\n");

        assert!(rendered.contains("src/main.rs"));
        assert!(rendered.contains("15"));
        assert!(rendered.contains('+'));
        assert!(rendered.contains("let port = 8080"));
        assert!(!rendered.contains("@@"));
    }

    #[test]
    fn renders_renamed_path_pair_on_one_line() {
        let changes = vec![FileChange {
            path: PathBuf::from("src/new.rs"),
            previous_path: Some(PathBuf::from("src/old.rs")),
            kind: ChangeKind::Renamed,
            is_directory: false,
            detail: None,
        }];

        let rendered = render_changes_panel(&changes, 0, 0, &BTreeSet::new(), 80, 8).join("\n");

        assert!(rendered.contains("from"));
        assert!(rendered.contains("src/old.rs"));
        assert!(rendered.contains("to"));
        assert!(rendered.contains("src/new.rs"));
    }

    #[test]
    fn deleted_file_does_not_render_deleted_diff_lines() {
        let changes = vec![FileChange {
            path: PathBuf::from("src/old.rs"),
            previous_path: None,
            kind: ChangeKind::Deleted,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: Some("line 1".to_string()),
                preview: vec!["- removed code".to_string()],
                truncated: false,
            }),
        }];

        let rendered = render_changes_panel(&changes, 0, 0, &BTreeSet::new(), 80, 8).join("\n");

        assert!(rendered.contains("file deleted"));
        assert!(!rendered.contains("removed code"));
    }

    #[test]
    fn scrolls_inside_single_large_change_block() {
        let changes = vec![FileChange {
            path: PathBuf::from("src/large.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: Some("line 1".to_string()),
                preview: vec![
                    "+ one".to_string(),
                    "+ two".to_string(),
                    "+ three".to_string(),
                    "+ four".to_string(),
                    "+ five".to_string(),
                    "+ six".to_string(),
                    "+ seven".to_string(),
                ],
                truncated: false,
            }),
        }];

        let top = render_changes_panel(&changes, 0, 0, &BTreeSet::new(), 80, 6).join("\n");
        let scrolled = render_changes_panel(&changes, 0, 4, &BTreeSet::new(), 80, 6).join("\n");

        assert!(top.contains("src/large.rs"));
        assert_ne!(top, scrolled);
        assert!(scrolled.contains("one") || scrolled.contains("two") || scrolled.contains("three"));
    }
}
