//! Logs panel rendering.

use rumon_shared::{LogEntry, LogKind};
use std::time::UNIX_EPOCH;

use crate::theme::{ColorToken, bold_paint, dim, paint};

/// Renders process and system logs.
#[must_use]
pub fn render_logs_panel(
    logs: &[LogEntry],
    _command: &str,
    scroll: usize,
    height: usize,
    show_timestamp: bool,
) -> Vec<String> {
    let mut lines = vec![bold_paint(ColorToken::Info, "Logs")];
    if logs.is_empty() {
        lines.push(String::new());
        lines.push(dim("No logs yet"));
        lines.push(dim("Command output will appear here."));
        return fit_height(lines, height);
    }

    let available = height.saturating_sub(lines.len());
    let start = scroll_start(logs.len(), scroll, available);
    for entry in logs.iter().skip(start).take(available) {
        lines.push(format_log(entry, show_timestamp));
    }

    fit_height(lines, height)
}

fn format_log(entry: &LogEntry, show_timestamp: bool) -> String {
    let message = sanitize_terminal_output(&entry.message);
    let (icon, token) = match entry.kind {
        LogKind::Stdout => (">", ColorToken::Info),
        LogKind::Stderr => ("!", ColorToken::Error),
        LogKind::System => ("◆", ColorToken::Info),
    };
    if entry.kind == LogKind::System && is_restart_message(&message) {
        return paint(ColorToken::Warning, format!("◆ {message}"));
    }
    let source = paint(token, icon);
    if show_timestamp {
        format!(
            "{} {} {}",
            dim(format!("[{}]", timestamp_label(entry))),
            source,
            paint(ColorToken::Text, message)
        )
    } else {
        format!("{source}  {}", paint(ColorToken::Text, message))
    }
}

fn scroll_start(log_count: usize, scroll: usize, available: usize) -> usize {
    if log_count == 0 || available == 0 {
        return 0;
    }
    if scroll >= log_count.saturating_sub(1) {
        log_count.saturating_sub(available)
    } else {
        scroll.min(log_count.saturating_sub(1))
    }
}

fn is_restart_message(message: &str) -> bool {
    message.contains("restarting") || message.contains("restart")
}

fn sanitize_terminal_output(value: &str) -> String {
    let without_escape = strip_ansi_sequences(value);
    without_escape
        .chars()
        .filter(|character| !character.is_control() || *character == '\t')
        .collect()
}

fn strip_ansi_sequences(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '\u{1b}' {
            consume_escape_sequence(&mut chars);
        } else {
            output.push(character);
        }
    }
    output
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

fn timestamp_label(entry: &LogEntry) -> String {
    entry.timestamp.duration_since(UNIX_EPOCH).map_or_else(
        |_| "00:00:00".to_string(),
        |duration| {
            let seconds = duration.as_secs() % 86_400;
            let hours = seconds / 3_600;
            let minutes = (seconds % 3_600) / 60;
            let seconds = seconds % 60;
            format!("{hours:02}:{minutes:02}:{seconds:02}")
        },
    )
}

fn fit_height(mut lines: Vec<String>, height: usize) -> Vec<String> {
    lines.truncate(height);
    while lines.len() < height {
        lines.push(String::new());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::render_logs_panel;
    use rumon_shared::{LogEntry, LogKind};

    #[test]
    fn stdout_restart_text_is_not_a_marker() {
        let logs = vec![LogEntry::new(LogKind::Stdout, "q quit r restart")];

        let rendered = render_logs_panel(&logs, "cargo run", 0, 6, false);

        assert!(
            rendered
                .iter()
                .any(|line| line.contains("q quit r restart"))
        );
        assert!(!rendered.iter().any(|line| line.starts_with('◆')));
    }

    #[test]
    fn strips_ansi_sequences_from_process_logs() {
        let logs = vec![LogEntry::new(LogKind::Stdout, "\u{1b}[Hhello\u{1b}[J")];

        let rendered = render_logs_panel(&logs, "cargo run", 0, 6, false);

        assert!(rendered.iter().any(|line| line.contains("hello")));
    }

    #[test]
    fn does_not_render_command_header() {
        let logs = vec![LogEntry::new(LogKind::Stdout, "10")];

        let rendered = render_logs_panel(&logs, "echo 10", 0, 6, false);

        assert!(!rendered.iter().any(|line| line.contains("echo 10")));
        assert!(rendered.iter().any(|line| line.contains('>')));
        assert!(rendered.iter().any(|line| line.contains("\u{1b}[")));
    }
}
