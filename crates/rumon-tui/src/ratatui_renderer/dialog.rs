//! Centered overlay dialogs.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

use super::style::{style, styled};

pub(super) fn render_dialog(
    frame: &mut Frame<'_>,
    area: Rect,
    title: &str,
    lines: Vec<Line<'static>>,
) {
    let width = area.width.saturating_mul(62).saturating_div(100).max(42);
    let height = u16::try_from(lines.len()).unwrap_or(0).saturating_add(2);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    let rect = Rect::new(x, y, width.min(area.width), height.min(area.height));
    frame.render_widget(Clear, rect);
    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .border_style(style(Color::Yellow)),
    );
    frame.render_widget(paragraph, rect);
}

pub(super) fn help_lines() -> Vec<Line<'static>> {
    vec![
        shortcut_line("q", "quit"),
        shortcut_line("r", "restart"),
        shortcut_line("c", "clear logs"),
        shortcut_line("/", "search"),
        shortcut_line("tab", "switch panel"),
        shortcut_line("enter", "expand/collapse"),
        shortcut_line("esc", "close dialog"),
    ]
}

pub(super) fn search_lines(query: &str) -> Vec<Line<'static>> {
    vec![Line::from(vec![
        styled("Search: ", Color::Cyan),
        styled(query.to_string(), Color::White),
    ])]
}

fn shortcut_line(key: &'static str, label: &'static str) -> Line<'static> {
    Line::from(vec![
        styled(key, Color::Cyan),
        Span::raw("  "),
        styled(label, Color::White),
    ])
}
