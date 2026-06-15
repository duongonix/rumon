//! Ratatui dashboard renderer.

mod changes;
mod chrome;
mod dialog;
mod logs;
mod style;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders};

use crate::app::TuiApp;

use self::changes::render_changes;
use self::chrome::{render_footer, render_status, render_too_small};
use self::dialog::{help_lines, render_dialog, search_lines};
use self::logs::render_logs;
use self::style::{PANEL_BORDER, style};

/// Renders the full Rumon dashboard.
pub fn render_dashboard(frame: &mut Frame<'_>, app: &TuiApp, _command: &str) {
    let area = frame.area();
    if area.width < 80 || area.height < 20 {
        render_too_small(frame, area);
        return;
    }

    let root = Block::default()
        .borders(Borders::ALL)
        .border_style(style(PANEL_BORDER));
    frame.render_widget(root, area);

    let inner = area.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 0,
    });
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(2),
        ])
        .split(inner);

    render_status(frame, rows[0], &app.state);
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);
    render_changes(frame, body[0], app);
    render_logs(frame, body[1], app);
    render_footer(frame, rows[2]);

    if app.help_visible {
        render_dialog(frame, area, "Help", help_lines());
    } else if let Some(query) = &app.search_query {
        render_dialog(frame, area, "Search", search_lines(query));
    }
}

fn panel_sections(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area)
}

#[cfg(test)]
mod tests {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use rumon_shared::{AppState, ChangeDetail, ChangeKind, FileChange, LogEntry, LogKind};

    use super::render_dashboard;
    use crate::focus::FocusTarget;
    use crate::{TuiApp, TuiConfig};

    #[test]
    fn renders_dashboard_with_ratatui_backend() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let app = TuiApp::new(AppState::default(), TuiConfig::default());

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let content = buffer
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(content.contains("Rumon"));
        assert!(content.contains(&format!("({})", env!("CARGO_PKG_VERSION"))));
        assert!(content.contains("Changes"));
        assert!(content.contains("Logs"));
        assert!(!content.contains("echo 10"));
    }

    #[test]
    fn renders_single_panel_titles_and_small_stdout_prompt() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut state = AppState::default();
        state.logs.push(LogEntry::new(LogKind::Stdout, "10"));
        let app = TuiApp::new(state, TuiConfig::default());

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert_eq!(content.matches("Changes").count(), 1);
        assert_eq!(content.matches("Logs").count(), 1);
        assert!(content.contains("›"));
        assert!(!content.contains("> 10"));
    }

    #[test]
    fn keeps_panel_headers_visible_while_scrolled() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let state = AppState {
            changes: (0..8)
                .map(|index| FileChange {
                    path: format!("src/file_{index}.rs").into(),
                    previous_path: None,
                    kind: ChangeKind::Modified,
                    is_directory: false,
                    detail: Some(ChangeDetail::Text {
                        location: Some("line 20 col 1".to_string()),
                        preview: vec!["+ let value = 1;".to_string()],
                        truncated: false,
                    }),
                })
                .collect(),
            logs: (0..40)
                .map(|index| LogEntry::new(LogKind::Stdout, format!("line {index}")))
                .collect(),
            ..AppState::default()
        };
        let mut app = TuiApp::new(state, TuiConfig::default());
        app.change_scroll = 12;
        app.log_scroll = 20;
        app.focus = FocusTarget::Logs;

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert_eq!(content.matches("Changes").count(), 1);
        assert_eq!(content.matches("Logs").count(), 1);
        assert!(content.contains("(8 files)"));
    }

    #[test]
    fn renders_diff_line_numbers_from_location() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut state = AppState::default();
        state.changes.push(FileChange {
            path: "src/al.rs".into(),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: Some("line 7 col 1".to_string()),
                preview: vec![
                    "- let b = 2;".to_string(),
                    "+".to_string(),
                    "+ let c = 3;".to_string(),
                ],
                truncated: false,
            }),
        });
        let app = TuiApp::new(state, TuiConfig::default());

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let content = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(ratatui::buffer::Cell::symbol)
            .collect::<String>();
        assert!(content.contains("   7 -| let b = 2;"));
        assert!(content.contains("   8 +|"));
        assert!(content.contains("   9 +| let c = 3;"));
    }

    #[test]
    fn change_cards_fill_the_changes_panel_width() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut state = AppState::default();
        state.changes.push(FileChange {
            path: "src/b.rs".into(),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: Some(ChangeDetail::Text {
                location: Some("line 9 col 1".to_string()),
                preview: vec!["+ let e = 5;".to_string()],
                truncated: false,
            }),
        });
        let app = TuiApp::new(state, TuiConfig::default());

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let row = (0..buffer.area.height)
            .find_map(|y| {
                let line = (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>();
                line.contains("[M]").then_some(line)
            })
            .expect("change card row");
        let row_chars = row.chars().collect::<Vec<_>>();
        let badge = row.find("[M]").expect("badge");
        let badge = row[..badge].chars().count();
        let card_right_border = row_chars
            .iter()
            .enumerate()
            .skip(badge)
            .find_map(|(index, character)| (*character == '│').then_some(index))
            .expect("card right border");
        let panel_right_border = row_chars
            .iter()
            .enumerate()
            .skip(card_right_border + 1)
            .find_map(|(index, character)| (*character == '│').then_some(index))
            .expect("changes panel right border");
        assert_eq!(panel_right_border, card_right_border + 1);
    }

    #[test]
    fn renders_balanced_panels() {
        let backend = TestBackend::new(120, 30);
        let mut terminal = Terminal::new(backend).expect("terminal");
        let app = TuiApp::new(AppState::default(), TuiConfig::default());

        terminal
            .draw(|frame| render_dashboard(frame, &app, "echo 10"))
            .expect("draw");

        let buffer = terminal.backend().buffer();
        let title_row = (0..buffer.area.height)
            .find_map(|y| {
                let line = (0..buffer.area.width)
                    .map(|x| buffer[(x, y)].symbol())
                    .collect::<String>();
                (line.contains("Changes") && line.contains("Logs")).then_some(line)
            })
            .expect("title row");
        let changes = title_row.find("Changes").expect("changes title");
        let logs = title_row.find("Logs").expect("logs title");
        assert!((55..=65).contains(&(logs - changes)));
    }
}
