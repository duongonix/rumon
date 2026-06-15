//! Overlay panel rendering.

use crate::theme::{ColorToken, bold_paint, dim, paint};

/// Renders the help overlay content.
#[must_use]
pub fn render_help_panel() -> Vec<String> {
    vec![
        bold_paint(ColorToken::Info, "Help"),
        help("q", "Quit"),
        help("r", "Restart process"),
        help("c", "Clear logs"),
        help("/", "Search active panel"),
        help("tab", "Switch panel"),
        help("?", "Toggle help"),
    ]
}

fn help(key: &str, label: &str) -> String {
    format!("{} {}", paint(ColorToken::Info, key), dim(label))
}
