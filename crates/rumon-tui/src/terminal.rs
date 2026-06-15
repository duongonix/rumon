//! Terminal dashboard rendering.

use std::env;
use std::process::Command;

use crate::app::TuiApp;
use crate::focus::FocusTarget;
use crate::layout::split_layout;
use crate::panels::{render_changes_panel, render_logs_panel};
use crate::theme::{ColorToken, bold_paint, paint};
use crate::widgets::{render_footer, render_help_dialog, render_search_dialog, render_status_bar};

/// Returns terminal dimensions with a stable fallback.
#[must_use]
pub fn terminal_size() -> (u16, u16) {
    terminal_size_from_env()
        .or_else(terminal_size_from_platform)
        .unwrap_or((120, 30))
}

fn terminal_size_from_env() -> Option<(u16, u16)> {
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())?;
    let height = env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())?;
    valid_size(width, height)
}

#[cfg(windows)]
fn terminal_size_from_platform() -> Option<(u16, u16)> {
    let output = Command::new("cmd").args(["/C", "mode CON"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_mode_con_output(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(not(windows))]
fn terminal_size_from_platform() -> Option<(u16, u16)> {
    let output = Command::new("stty").arg("size").output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_stty_size_output(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn parse_mode_con_output(output: &str) -> Option<(u16, u16)> {
    let mut width = None;
    let mut height = None;
    for line in output.lines() {
        let normalized = line.trim().to_ascii_lowercase();
        if normalized.starts_with("columns:") {
            width = number_after_colon(line);
        } else if normalized.starts_with("lines:") {
            height = number_after_colon(line);
        }
    }
    valid_size(width?, height?)
}

#[cfg(not(windows))]
fn parse_stty_size_output(output: &str) -> Option<(u16, u16)> {
    let mut parts = output.split_whitespace();
    let height = parts.next()?.parse().ok()?;
    let width = parts.next()?.parse().ok()?;
    valid_size(width, height)
}

#[cfg(windows)]
fn number_after_colon(line: &str) -> Option<u16> {
    line.split_once(':')?.1.trim().parse().ok()
}

fn valid_size(width: u16, height: u16) -> Option<(u16, u16)> {
    (width > 0 && height > 0).then_some((width, height))
}

/// Renders the full dashboard as terminal text.
#[must_use]
pub fn render_dashboard(app: &TuiApp, width: u16, height: u16, command: &str) -> String {
    let layout = split_layout(width, height, app.config.left_panel_width);
    if layout.too_small {
        return "Terminal too small\nMinimum recommended size: 100 x 30".to_string();
    }

    let mut lines = Vec::new();
    lines.push(top_border(width));
    lines.push(wrap_border_line(
        &render_status_bar(&app.state, None, app.state.changes.len()),
        usize::from(width),
    ));
    lines.push(separator(width));

    let dialog = active_dialog(app, usize::from(width));
    let dialog_height = dialog.as_ref().map_or(0, Vec::len);
    let changes_height = if width < 100 {
        usize::from(layout.changes.height.saturating_sub(2))
    } else {
        usize::from(layout.changes.height.saturating_sub(2)).saturating_sub(dialog_height)
    };
    let logs_height =
        usize::from(layout.logs.height.saturating_sub(2)).saturating_sub(dialog_height);
    let changes = render_changes_panel(
        &app.state.changes,
        app.selected_change,
        app.change_scroll,
        &app.collapsed_changes,
        usize::from(layout.changes.width).saturating_sub(4),
        changes_height,
    );
    let logs = render_logs_panel(
        &app.state.logs,
        command,
        app.log_scroll,
        logs_height,
        app.config.show_timestamp,
    );

    if width < 100 {
        let inner_width = usize::from(width).saturating_sub(4);
        let changes = with_title_underline(changes, inner_width, app.focus == FocusTarget::Changes);
        let logs = with_title_underline(logs, inner_width, app.focus == FocusTarget::Logs);
        lines.extend(frame_panel("Changes", &changes, usize::from(width)));
        lines.extend(frame_panel("Logs", &logs, usize::from(width)));
    } else {
        let changes = with_title_underline(
            changes,
            usize::from(layout.changes.width).saturating_sub(3),
            app.focus == FocusTarget::Changes,
        );
        let logs = with_title_underline(
            logs,
            usize::from(layout.logs.width).saturating_sub(4),
            app.focus == FocusTarget::Logs,
        );
        lines.extend(join_panels(
            &changes,
            usize::from(layout.changes.width),
            &logs,
            usize::from(layout.logs.width),
        ));
    }

    if let Some(dialog) = dialog {
        lines.extend(dialog);
    }

    lines.push(separator(width));
    lines.push(wrap_border_line(&render_footer(), usize::from(width)));
    lines.push(bottom_border(width));
    lines.join("\n")
}

fn join_panels(
    left: &[String],
    left_width: usize,
    right: &[String],
    right_width: usize,
) -> Vec<String> {
    let height = left.len().max(right.len());
    let left_inner = left_width.saturating_sub(3);
    let right_inner = right_width.saturating_sub(4);
    (0..height)
        .map(|index| {
            format!(
                "{} {} {} {} {}",
                border("│"),
                fit(left.get(index).map_or("", String::as_str), left_inner),
                border("│"),
                fit(right.get(index).map_or("", String::as_str), right_inner),
                border("│")
            )
        })
        .collect()
}

fn frame_panel(title: &str, lines: &[String], width: usize) -> Vec<String> {
    let inner = width.saturating_sub(4);
    let mut rendered = vec![
        border(format!("├{}┤", "─".repeat(width.saturating_sub(2)))),
        wrap_border_line(&bold_paint(ColorToken::Info, title), width),
    ];
    rendered.extend(
        lines
            .iter()
            .map(|line| wrap_border_line(&fit(line, inner), width)),
    );
    rendered
}

fn active_dialog(app: &TuiApp, width: usize) -> Option<Vec<String>> {
    if app.help_visible {
        return Some(render_dialog("Help", &render_help_dialog(), width));
    }
    app.search_query
        .as_deref()
        .map(|query| render_dialog("Search", &render_search_dialog(Some(query)), width))
}

fn render_dialog(title: &str, body: &[String], width: usize) -> Vec<String> {
    let max_width = width.saturating_sub(8).max(20);
    let min_width = 42.min(max_width);
    let dialog_width = width
        .saturating_mul(62)
        .saturating_div(100)
        .clamp(min_width, max_width);
    let inner = dialog_width.saturating_sub(2);
    let mut rows = Vec::with_capacity(body.len() + 4);
    rows.push(center_dialog_line(
        &paint(ColorToken::BorderActive, format!("┌{}┐", "─".repeat(inner))),
        width,
        dialog_width,
    ));
    rows.push(center_dialog_line(
        &format!(
            "{}{}{}",
            paint(ColorToken::BorderActive, "│"),
            fit(&bold_paint(ColorToken::Info, title), inner),
            paint(ColorToken::BorderActive, "│")
        ),
        width,
        dialog_width,
    ));
    rows.push(center_dialog_line(
        &paint(ColorToken::BorderActive, format!("├{}┤", "─".repeat(inner))),
        width,
        dialog_width,
    ));
    rows.extend(body.iter().map(|line| {
        center_dialog_line(
            &format!(
                "{}{}{}",
                paint(ColorToken::BorderActive, "│"),
                fit(line, inner),
                paint(ColorToken::BorderActive, "│")
            ),
            width,
            dialog_width,
        )
    }));
    rows.push(center_dialog_line(
        &paint(ColorToken::BorderActive, format!("└{}┘", "─".repeat(inner))),
        width,
        dialog_width,
    ));
    rows
}

fn center_dialog_line(dialog_line: &str, width: usize, dialog_width: usize) -> String {
    let available = width.saturating_sub(4);
    let left = available.saturating_sub(dialog_width) / 2;
    let right = available.saturating_sub(dialog_width + left);
    format!(
        "{} {}{}{} {}",
        border("│"),
        " ".repeat(left),
        dialog_line,
        " ".repeat(right),
        border("│")
    )
}

fn with_title_underline(mut lines: Vec<String>, width: usize, active: bool) -> Vec<String> {
    let target_len = lines.len();
    let underline = title_underline(width, active);
    if lines.is_empty() {
        lines.push(underline);
    } else {
        lines.insert(1, underline);
    }
    lines.truncate(target_len);
    lines
}

fn top_border(width: u16) -> String {
    border(format!(
        "┌{}┐",
        "─".repeat(usize::from(width).saturating_sub(2))
    ))
}

fn separator(width: u16) -> String {
    separator_usize(usize::from(width))
}

fn bottom_border(width: u16) -> String {
    border(format!(
        "└{}┘",
        "─".repeat(usize::from(width).saturating_sub(2))
    ))
}

fn wrap_border_line(value: &str, width: usize) -> String {
    format!(
        "{} {} {}",
        border("│"),
        fit(value, width.saturating_sub(4)),
        border("│")
    )
}

fn fit(value: &str, width: usize) -> String {
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

fn separator_usize(width: usize) -> String {
    border(format!("├{}┤", "─".repeat(width.saturating_sub(2))))
}

fn border(value: impl AsRef<str>) -> String {
    paint(ColorToken::Border, value.as_ref())
}

fn title_underline(width: usize, active: bool) -> String {
    if active {
        paint(ColorToken::Warning, "─".repeat(width))
    } else {
        paint(ColorToken::Border, "─".repeat(width))
    }
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

#[cfg(test)]
mod tests {
    use super::render_dashboard;
    use crate::{TuiApp, TuiConfig};
    use rumon_shared::AppState;

    #[test]
    fn renders_main_panel_titles() {
        let app = TuiApp::new(AppState::default(), TuiConfig::default());
        let output = render_dashboard(&app, 120, 30, "cargo run");

        assert!(output.contains("Changes"));
        assert!(output.contains("Logs"));
        assert!(output.contains("quit"));
    }
}
