//! Dashboard chrome widgets.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use rumon_shared::AppState;

use super::style::{PANEL_BORDER, border_line, style, styled};

pub(super) fn render_too_small(frame: &mut Frame<'_>, area: Rect) {
    let paragraph = Paragraph::new(vec![
        Line::from(styled("Terminal too small", Color::Yellow)),
        Line::from(styled("Minimum recommended size: 100 x 30", Color::Gray)),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).title("Rumon"));
    frame.render_widget(paragraph, area);
}

pub(super) fn render_status(frame: &mut Frame<'_>, area: Rect, state: &AppState) {
    let status = match state.process {
        rumon_shared::ProcessState::Running => styled("● running", Color::Green),
        rumon_shared::ProcessState::Restarting => styled("● restarting", Color::Yellow),
        rumon_shared::ProcessState::Failed => styled("● failed", Color::Red),
        rumon_shared::ProcessState::Starting => styled("● starting", Color::Cyan),
        rumon_shared::ProcessState::Stopped => styled("● stopped", Color::Blue),
    };
    let line = Line::from(vec![
        styled("◈ Rumon", Color::Cyan),
        Span::raw(" "),
        styled(format!("({})", env!("CARGO_PKG_VERSION")), Color::Yellow),
        Span::raw("   "),
        styled("dev", Color::Gray),
        Span::raw("   "),
        status,
        Span::raw("   "),
        styled("profile:", Color::Gray),
        Span::raw(" "),
        styled("none", Color::White),
        Span::raw("   "),
        styled("restarts:", Color::Gray),
        Span::raw(" "),
        styled(state.restart_count.to_string(), Color::Yellow),
        Span::raw("   "),
        styled("watch:", Color::Gray),
        Span::raw(" "),
        styled(format!("{} files", state.changes.len()), Color::Cyan),
    ]);
    frame.render_widget(
        Paragraph::new(vec![line, Line::from(border_line(area.width))]),
        area,
    );
}

pub(super) fn render_footer(frame: &mut Frame<'_>, area: Rect) {
    let line = Line::from(vec![
        styled("q", Color::Cyan),
        styled(" quit", Color::White),
        Span::raw("    "),
        styled("r", Color::Cyan),
        styled(" restart", Color::White),
        Span::raw("    "),
        styled("c", Color::Cyan),
        styled(" clear logs", Color::White),
        Span::raw("    "),
        styled("/", Color::Cyan),
        styled(" search", Color::White),
        Span::raw("    "),
        styled("tab", Color::Cyan),
        styled(" switch panel", Color::White),
        Span::raw("        "),
        styled("↑/↓", Color::Cyan),
        styled(" navigate", Color::White),
        Span::raw("    "),
        styled("enter", Color::Cyan),
        styled(" expand/collapse", Color::White),
    ]);
    frame.render_widget(
        Paragraph::new(vec![Line::from(border_line(area.width)), line]),
        area,
    );
}

pub(super) fn panel_block() -> Block<'static> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(style(PANEL_BORDER))
}
