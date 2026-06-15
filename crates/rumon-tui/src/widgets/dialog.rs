//! Centered dialog widgets.

use crate::theme::{ColorToken, bold_paint, dim, paint};

/// Renders a search dialog body.
#[must_use]
pub fn render_search_dialog(query: Option<&str>) -> Vec<String> {
    let value = query.filter(|value| !value.is_empty()).unwrap_or("");
    let input = if value.is_empty() {
        dim("type to filter")
    } else {
        paint(ColorToken::Text, value)
    };
    vec![
        format!(
            "{} {}",
            paint(ColorToken::Info, "/"),
            bold_paint(ColorToken::Text, "Search")
        ),
        format!("{} {}", dim("query"), input),
        format!("{} {}", paint(ColorToken::Info, "esc"), dim("close")),
    ]
}

/// Renders a help dialog body.
#[must_use]
pub fn render_help_dialog() -> Vec<String> {
    vec![
        format!("{} {}", paint(ColorToken::Info, "q"), dim("quit")),
        format!(
            "{} {}",
            paint(ColorToken::Info, "r"),
            dim("restart process")
        ),
        format!("{} {}", paint(ColorToken::Info, "c"), dim("clear logs")),
        format!("{} {}", paint(ColorToken::Info, "/"), dim("search")),
        format!("{} {}", paint(ColorToken::Info, "tab"), dim("switch panel")),
        format!("{} {}", paint(ColorToken::Info, "↑/↓"), dim("navigate")),
        format!(
            "{} {}",
            paint(ColorToken::Info, "enter"),
            dim("expand/collapse")
        ),
        format!("{} {}", paint(ColorToken::Info, "esc"), dim("close dialog")),
    ]
}
