//! Search widget.

use crate::theme::{ColorToken, dim, paint};

/// Renders search input content.
#[must_use]
pub fn render_search(query: Option<&str>) -> String {
    let value = match query.filter(|value| !value.is_empty()) {
        Some(value) => paint(ColorToken::Text, value),
        None => dim("type to filter"),
    };
    format!("{} {}", paint(ColorToken::Info, "Search:"), value)
}
