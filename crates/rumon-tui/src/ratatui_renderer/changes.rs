//! Changes panel renderer.

use std::collections::BTreeSet;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use rumon_shared::{ChangeDetail, ChangeKind, FileChange};

use super::chrome::panel_block;
use super::panel_sections;
use super::style::{CARD_BORDER, PANEL_BORDER, fit, format_size, style, styled};
use crate::app::TuiApp;
use crate::focus::FocusTarget;

pub(super) fn render_changes(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let active = app.focus == FocusTarget::Changes;
    let block = panel_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let sections = panel_sections(inner);
    let header = vec![
        Line::from(vec![
            styled("Changes", Color::Cyan),
            Span::raw(" "),
            styled(format!("({} files)", app.state.changes.len()), Color::Gray),
        ]),
        Line::from(styled(
            "─".repeat(usize::from(inner.width)),
            if active { Color::Yellow } else { PANEL_BORDER },
        )),
    ];
    frame.render_widget(Paragraph::new(header), sections[0]);

    let lines = change_lines(
        &app.state.changes,
        app.selected_change,
        &app.collapsed_changes,
        usize::from(inner.width),
    );
    let scroll = u16::try_from(app.change_scroll.min(usize::from(u16::MAX))).unwrap_or(u16::MAX);
    let paragraph = Paragraph::new(lines)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, sections[1]);
}

fn change_lines(
    changes: &[FileChange],
    selected: usize,
    collapsed: &BTreeSet<usize>,
    width: usize,
) -> Vec<Line<'static>> {
    if changes.is_empty() {
        return vec![
            Line::from(""),
            Line::from(styled("  No file changes yet", Color::Gray)),
            Line::from(styled("  Waiting for filesystem events...", Color::Gray)),
        ];
    }

    let mut lines = Vec::new();
    let card_width = width.max(20);
    for (index, change) in changes.iter().enumerate() {
        let border = if index == selected {
            Color::Cyan
        } else {
            CARD_BORDER
        };
        let inner = card_width.saturating_sub(2);
        lines.push(Line::from(styled(
            format!("┌{}┐", "─".repeat(inner)),
            border,
        )));
        lines.push(change_header_line(change, inner));
        lines.push(change_summary_line(
            change,
            collapsed.contains(&index),
            inner,
        ));
        lines.push(Line::from(styled(
            format!("├{}┤", "─".repeat(inner)),
            border,
        )));
        if !collapsed.contains(&index) {
            lines.extend(change_detail_lines(change, inner));
        }
        lines.push(Line::from(styled(
            format!("└{}┘", "─".repeat(inner)),
            border,
        )));
        lines.push(Line::from(""));
    }
    lines
}

fn change_header_line(change: &FileChange, width: usize) -> Line<'static> {
    let badge = match change.kind {
        ChangeKind::Created if change.is_directory => ("[F]", Color::Yellow),
        ChangeKind::Created => ("[A]", Color::Green),
        ChangeKind::Modified => ("[M]", Color::Cyan),
        ChangeKind::Deleted => ("[D]", Color::Red),
        ChangeKind::Renamed => ("[R]", Color::Yellow),
    };
    Line::from(vec![
        styled("│ ", CARD_BORDER),
        styled(badge.0, badge.1),
        Span::raw("  "),
        styled(
            fit(&header_path(change), width.saturating_sub(6)),
            Color::White,
        ),
        styled("│", CARD_BORDER),
    ])
}

fn change_summary_line(change: &FileChange, collapsed: bool, width: usize) -> Line<'static> {
    let chevron = if collapsed { "▸" } else { "▾" };
    let (added, removed) = diff_stats(change);
    let used = 6
        + kind_label(change).chars().count()
        + 1
        + stats_visible_width(added, removed)
        + 1
        + chevron.chars().count();
    let trailing = width.saturating_sub(used);
    let mut spans = vec![
        styled("│", CARD_BORDER),
        Span::raw("      "),
        styled(kind_label(change), Color::Yellow),
        Span::raw(" "),
    ];
    push_stats_spans(&mut spans, added, removed);
    spans.push(Span::raw(" "));
    spans.push(styled(chevron, Color::White));
    spans.push(Span::raw(" ".repeat(trailing)));
    spans.push(styled("│", CARD_BORDER));
    Line::from(spans)
}

fn change_detail_lines(change: &FileChange, width: usize) -> Vec<Line<'static>> {
    match change.kind {
        ChangeKind::Deleted => return vec![card_line("  file deleted", width, Color::Red)],
        ChangeKind::Renamed => {
            let previous = change
                .previous_path
                .as_ref()
                .map_or_else(|| display(&change.path), |path| display(path));
            return vec![card_line(
                &format!("  from {previous} to {}", display(&change.path)),
                width,
                Color::Yellow,
            )];
        }
        ChangeKind::Created if change.is_directory => {
            return vec![card_line("  folder created", width, Color::Yellow)];
        }
        _ => {}
    }

    match &change.detail {
        Some(ChangeDetail::Text {
            location,
            preview,
            truncated,
        }) => {
            let start_line = location.as_deref().and_then(first_number).unwrap_or(1);
            let mut lines: Vec<_> = preview
                .iter()
                .flat_map(|line| line.lines())
                .take(10)
                .enumerate()
                .map(|(offset, line)| diff_line(start_line + offset, line, width))
                .collect();
            if *truncated {
                lines.push(card_line("  diff preview truncated", width, Color::Gray));
            }
            lines
        }
        Some(ChangeDetail::Media {
            kind,
            mime_type,
            size_bytes,
            metadata,
        }) => {
            let mut lines = vec![
                card_line(&format!("  {kind}"), width, Color::Cyan),
                card_line(&format!("  {mime_type}"), width, Color::Cyan),
                card_line(
                    &format!("  {}", format_size(*size_bytes)),
                    width,
                    Color::Yellow,
                ),
            ];
            lines.extend(
                metadata
                    .iter()
                    .take(4)
                    .map(|line| card_line(&format!("  {line}"), width, Color::White)),
            );
            lines
        }
        Some(ChangeDetail::Binary {
            previous_size,
            current_size,
            hash_changed,
        }) => {
            let mut lines = vec![card_line("  metadata changed", width, Color::Gray)];
            if let (Some(previous), Some(current)) = (previous_size, current_size) {
                lines.push(card_line(
                    &format!(
                        "  size: {} -> {}",
                        format_size(*previous),
                        format_size(*current)
                    ),
                    width,
                    Color::Yellow,
                ));
            }
            if *hash_changed {
                lines.push(card_line("  hash changed", width, Color::Yellow));
            }
            lines
        }
        Some(ChangeDetail::Deleted) => vec![card_line("  file deleted", width, Color::Red)],
        None => vec![card_line("  metadata changed", width, Color::Gray)],
    }
}

fn diff_line(line_number: usize, line: &str, width: usize) -> Line<'static> {
    let (mark, body, color, background) = match line.chars().next() {
        Some('+') => (
            "+",
            line.get(1..).unwrap_or_default().trim_start(),
            Color::Green,
            Some(Color::Rgb(5, 46, 22)),
        ),
        Some('-') => (
            "-",
            line.get(1..).unwrap_or_default().trim_start(),
            Color::Red,
            Some(Color::Rgb(69, 10, 10)),
        ),
        _ => (" ", line.trim_start(), Color::White, None),
    };
    let text = fit(&format!("{line_number:>4} {mark}| {body}"), width);
    let content_style = background.map_or_else(|| style(color), |bg| style(color).bg(bg));
    Line::from(vec![
        styled("│", CARD_BORDER),
        Span::styled(text, content_style),
        styled("│", CARD_BORDER),
    ])
}

fn card_line(text: &str, width: usize, color: Color) -> Line<'static> {
    Line::from(vec![
        styled("│", CARD_BORDER),
        styled(fit(text, width), color),
        styled("│", CARD_BORDER),
    ])
}

fn header_path(change: &FileChange) -> String {
    if change.kind == ChangeKind::Renamed {
        change
            .previous_path
            .as_ref()
            .map_or_else(|| display(&change.path), |path| display(path))
    } else {
        display(&change.path)
    }
}

fn display(path: &std::path::Path) -> String {
    rumon_shared::display_path(path)
}

fn kind_label(change: &FileChange) -> &'static str {
    match change.kind {
        ChangeKind::Created => "created",
        ChangeKind::Modified => "modified",
        ChangeKind::Deleted => "deleted",
        ChangeKind::Renamed => "renamed",
    }
}

fn diff_stats(change: &FileChange) -> (usize, usize) {
    let Some(ChangeDetail::Text { preview, .. }) = &change.detail else {
        return (0, 0);
    };
    let added = preview.iter().filter(|line| line.starts_with('+')).count();
    let removed = preview.iter().filter(|line| line.starts_with('-')).count();
    (added, removed)
}

fn stats_visible_width(added: usize, removed: usize) -> usize {
    if added > 0 || removed > 0 {
        format!("+{added} / -{removed}").chars().count()
    } else {
        0
    }
}

fn push_stats_spans(spans: &mut Vec<Span<'static>>, added: usize, removed: usize) {
    if added == 0 && removed == 0 {
        return;
    }
    spans.push(styled(format!("+{added}"), Color::Green));
    spans.push(styled(" / ", Color::Gray));
    spans.push(styled(format!("-{removed}"), Color::Red));
}

fn first_number(value: &str) -> Option<usize> {
    value
        .split(|character: char| !character.is_ascii_digit())
        .find(|part| !part.is_empty())
        .and_then(|part| part.parse().ok())
}
