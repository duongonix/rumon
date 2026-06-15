//! Logs panel renderer.

use std::time::UNIX_EPOCH;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use rumon_shared::{LogEntry, LogKind};

use super::chrome::panel_block;
use super::panel_sections;
use super::style::{PANEL_BORDER, styled};
use crate::app::TuiApp;
use crate::focus::FocusTarget;

pub(super) fn render_logs(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let active = app.focus == FocusTarget::Logs;
    let block = panel_block();
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let sections = panel_sections(inner);
    let header = vec![
        Line::from(styled("Logs", Color::Cyan)),
        Line::from(styled(
            "─".repeat(usize::from(inner.width)),
            if active { Color::Yellow } else { PANEL_BORDER },
        )),
    ];
    frame.render_widget(Paragraph::new(header), sections[0]);

    let mut lines = Vec::new();
    lines.extend(
        app.state
            .logs
            .iter()
            .map(|entry| log_line(entry, app.config.show_timestamp)),
    );
    if app.state.logs.is_empty() {
        lines.push(Line::from(styled("No logs yet", Color::Gray)));
    }
    let scroll = log_scroll(&app.state.logs, app.log_scroll, sections[1].height);
    let paragraph = Paragraph::new(lines)
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, sections[1]);
}

fn log_line(entry: &LogEntry, show_timestamp: bool) -> Line<'static> {
    let message = strip_controls(&entry.message);
    let (icon, color) = match entry.kind {
        LogKind::Stdout => ("›", Color::Cyan),
        LogKind::Stderr => ("!", Color::Red),
        LogKind::System => ("◆", Color::Yellow),
    };
    let mut spans = Vec::new();
    if show_timestamp {
        spans.push(styled(
            format!("[{}] ", timestamp_label(entry)),
            Color::Gray,
        ));
    }
    spans.push(styled(icon, color));
    spans.push(Span::raw(" "));
    spans.push(styled(message, Color::White));
    Line::from(spans)
}

fn log_scroll(logs: &[LogEntry], scroll: usize, height: u16) -> u16 {
    if logs.is_empty() {
        return 0;
    }
    let visible = usize::from(height);
    let max = logs.len().saturating_sub(visible);
    u16::try_from(scroll.min(max)).unwrap_or(u16::MAX)
}

fn timestamp_label(entry: &LogEntry) -> String {
    entry.timestamp.duration_since(UNIX_EPOCH).map_or_else(
        |_| "00:00:00".to_string(),
        |duration| {
            let seconds = duration.as_secs() % 86_400;
            format!(
                "{:02}:{:02}:{:02}",
                seconds / 3_600,
                (seconds % 3_600) / 60,
                seconds % 60
            )
        },
    )
}

fn strip_controls(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_control() || *character == '\t')
        .collect()
}
