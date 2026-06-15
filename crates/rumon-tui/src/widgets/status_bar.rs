//! Status bar widget.

use rumon_shared::{AppState, ProcessState};

use crate::theme::{ColorToken, bold_paint, dim, paint};

/// Renders the global status bar.
#[must_use]
pub fn render_status_bar(state: &AppState, profile: Option<&str>, watched_files: usize) -> String {
    format!(
        "{} {}  {}  {}   {} {}   {} {}   {} {} files",
        bold_paint(ColorToken::Info, "◈ Rumon"),
        paint(
            ColorToken::Warning,
            format!("({})", env!("CARGO_PKG_VERSION"))
        ),
        dim("dev"),
        process_label(&state.process),
        dim("profile:"),
        paint(ColorToken::Text, profile.unwrap_or("none")),
        dim("restarts:"),
        paint(ColorToken::Warning, state.restart_count.to_string()),
        dim("watch:"),
        paint(ColorToken::Info, watched_files.to_string())
    )
}

fn process_label(state: &ProcessState) -> String {
    match state {
        ProcessState::Starting => paint(ColorToken::Warning, "● starting"),
        ProcessState::Running => paint(ColorToken::Success, "● running"),
        ProcessState::Restarting => paint(ColorToken::Warning, "● restarting"),
        ProcessState::Stopped => paint(ColorToken::Muted, "● stopped"),
        ProcessState::Failed => paint(ColorToken::Error, "● failed"),
    }
}
