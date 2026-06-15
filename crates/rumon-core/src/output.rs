//! Plain terminal output for the Phase 1 runtime.

use rumon_shared::{
    AppEvent, ChangeKind, LogEntry, LogKind, ProcessEvent, WatchEvent, display_path,
};

/// Prints an application event in plain text mode.
pub fn print_plain_event(event: &AppEvent, quiet: bool) {
    if quiet {
        return;
    }

    match event {
        AppEvent::Watch(WatchEvent::Changed(change)) => {
            println!("{}", format_watch_change(change));
        }
        AppEvent::Watch(WatchEvent::Error(message)) | AppEvent::System(message) => {
            eprintln!("{} {}", paint(Color::Red, "✕ rumon"), message);
        }
        AppEvent::Process(ProcessEvent::Log(LogEntry {
            kind: LogKind::Stdout,
            message,
            ..
        })) => println!("{} {}", paint(Color::Cyan, "›"), message),
        AppEvent::Process(ProcessEvent::Log(LogEntry {
            kind: LogKind::Stderr,
            message,
            ..
        })) => eprintln!("{} {}", paint(Color::Red, "›"), message),
        AppEvent::Process(
            ProcessEvent::Log(LogEntry {
                kind: LogKind::System,
                ..
            })
            | ProcessEvent::Starting
            | ProcessEvent::Started
            | ProcessEvent::Restarting
            | ProcessEvent::Stopped
            | ProcessEvent::Exited(Some(0) | None),
        ) => {}
        AppEvent::Process(ProcessEvent::Exited(Some(code))) => {
            eprintln!("{} command exited with {code}", paint(Color::Red, "✕"));
        }
        AppEvent::Process(ProcessEvent::Failed(message)) => {
            eprintln!("{} process failed: {message}", paint(Color::Red, "✕"));
        }
    }
}

fn format_watch_change(change: &rumon_shared::FileChange) -> String {
    let (icon, label, color) = match change.kind {
        ChangeKind::Created => ("✚", "path created", Color::Green),
        ChangeKind::Modified => ("●", "path modified", Color::Yellow),
        ChangeKind::Deleted => ("✕", "path deleted", Color::Red),
        ChangeKind::Renamed => ("↻", "path renamed", Color::Magenta),
    };
    let headline = format!("{} {}", paint(color, icon), paint(color, label));
    if let Some(previous_path) = &change.previous_path {
        format!(
            "{headline}  {} {} {}",
            paint(Color::Muted, display_path(previous_path)),
            paint(Color::Muted, "→"),
            paint(Color::Text, display_path(&change.path))
        )
    } else {
        format!(
            "{headline}  {}",
            paint(Color::Text, display_path(&change.path))
        )
    }
}

#[derive(Clone, Copy)]
enum Color {
    Text,
    Muted,
    Red,
    Green,
    Yellow,
    Magenta,
    Cyan,
}

fn paint(color: Color, value: impl AsRef<str>) -> String {
    let code = match color {
        Color::Text => "97",
        Color::Muted => "90",
        Color::Red => "91",
        Color::Green => "92",
        Color::Yellow => "93",
        Color::Magenta => "95",
        Color::Cyan => "96",
    };
    format!("\x1b[{code}m{}\x1b[0m", value.as_ref())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rumon_shared::{ChangeKind, FileChange};

    use super::format_watch_change;

    #[test]
    fn formats_modified_path_without_workspace_prefix() {
        let line = format_watch_change(&FileChange {
            path: PathBuf::from("src/al.rs"),
            previous_path: None,
            kind: ChangeKind::Modified,
            is_directory: false,
            detail: None,
        });

        assert!(line.contains("path modified"));
        assert!(line.contains("src/al.rs"));
    }

    #[test]
    fn formats_rename_pair() {
        let line = format_watch_change(&FileChange {
            path: PathBuf::from("src/new.rs"),
            previous_path: Some(PathBuf::from("src/old.rs")),
            kind: ChangeKind::Renamed,
            is_directory: false,
            detail: None,
        });

        assert!(line.contains("src/old.rs"));
        assert!(line.contains("src/new.rs"));
    }
}
