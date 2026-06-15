//! Shared Ratatui styling helpers.

use ratatui::style::{Color, Style};
use ratatui::text::Span;

pub(super) const PANEL_BORDER: Color = Color::Rgb(36, 48, 68);
pub(super) const CARD_BORDER: Color = Color::Rgb(75, 85, 99);

pub(super) fn styled(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(text.into(), style(color))
}

pub(super) fn style(color: Color) -> Style {
    Style::default().fg(color)
}

pub(super) fn border_line(width: u16) -> String {
    "─".repeat(usize::from(width))
}

pub(super) fn fit(value: &str, width: usize) -> String {
    let mut output: String = value.chars().take(width).collect();
    let used = output.chars().count();
    if used < width {
        output.push_str(&" ".repeat(width - used));
    }
    output
}

pub(super) fn format_size(bytes: u64) -> String {
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
