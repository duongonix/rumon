//! TUI theme tokens.

/// Semantic color tokens used by the TUI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorToken {
    /// Background color.
    Background,
    /// Surface color.
    Surface,
    /// Border color.
    Border,
    /// Active border color.
    BorderActive,
    /// Primary text color.
    Text,
    /// Muted text color.
    Muted,
    /// Success color.
    Success,
    /// Error color.
    Error,
    /// Warning color.
    Warning,
    /// Informational color.
    Info,
    /// Added content color.
    Added,
    /// Removed content color.
    Removed,
    /// Changed content color.
    Changed,
    /// Added line background.
    AddedBackground,
    /// Removed line background.
    RemovedBackground,
    /// Diff hunk background.
    HunkBackground,
}

/// Theme palette expressed as hex strings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theme {
    /// Background.
    pub background: &'static str,
    /// Surface.
    pub surface: &'static str,
    /// Border.
    pub border: &'static str,
    /// Active border.
    pub border_active: &'static str,
    /// Text.
    pub text: &'static str,
    /// Muted text.
    pub muted: &'static str,
    /// Success.
    pub success: &'static str,
    /// Error.
    pub error: &'static str,
    /// Warning.
    pub warning: &'static str,
    /// Info.
    pub info: &'static str,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#0B1020",
            surface: "#111827",
            border: "#243044",
            border_active: "#38BDF8",
            text: "#E5E7EB",
            muted: "#94A3B8",
            success: "#22C55E",
            error: "#EF4444",
            warning: "#EAB308",
            info: "#06B6D4",
        }
    }
}

/// ANSI reset sequence.
pub const RESET: &str = "\x1b[0m";

/// Paints text with a semantic foreground color.
#[must_use]
pub fn paint(token: ColorToken, text: impl AsRef<str>) -> String {
    format!("{}{}{}", fg(token), text.as_ref(), RESET)
}

/// Paints text with dim emphasis.
#[must_use]
pub fn dim(text: impl AsRef<str>) -> String {
    format!("\x1b[2m{}{}", text.as_ref(), RESET)
}

/// Paints text with bold foreground color.
#[must_use]
pub fn bold_paint(token: ColorToken, text: impl AsRef<str>) -> String {
    format!("\x1b[1m{}{}{}", fg(token), text.as_ref(), RESET)
}

/// Paints text with foreground and background semantic colors.
#[must_use]
pub fn paint_on(foreground: ColorToken, background: ColorToken, text: impl AsRef<str>) -> String {
    format!(
        "{}{}{}{}",
        fg(foreground),
        bg(background),
        text.as_ref(),
        RESET
    )
}

/// Paints text with bold foreground and background semantic colors.
#[must_use]
pub fn bold_paint_on(
    foreground: ColorToken,
    background: ColorToken,
    text: impl AsRef<str>,
) -> String {
    format!(
        "\x1b[1m{}{}{}{}",
        fg(foreground),
        bg(background),
        text.as_ref(),
        RESET
    )
}

fn fg(token: ColorToken) -> &'static str {
    match token {
        ColorToken::Background => "\x1b[38;2;11;16;32m",
        ColorToken::Surface => "\x1b[38;2;17;24;39m",
        ColorToken::Border => "\x1b[38;2;36;48;68m",
        ColorToken::BorderActive => "\x1b[38;2;56;189;248m",
        ColorToken::Text => "\x1b[38;2;229;231;235m",
        ColorToken::Muted => "\x1b[38;2;148;163;184m",
        ColorToken::Success | ColorToken::Added => "\x1b[38;2;34;197;94m",
        ColorToken::Error | ColorToken::Removed => "\x1b[38;2;239;68;68m",
        ColorToken::Warning | ColorToken::Changed => "\x1b[38;2;234;179;8m",
        ColorToken::Info => "\x1b[38;2;6;182;212m",
        ColorToken::AddedBackground => "\x1b[38;2;20;83;45m",
        ColorToken::RemovedBackground => "\x1b[38;2;127;29;29m",
        ColorToken::HunkBackground => "\x1b[38;2;30;64;105m",
    }
}

fn bg(token: ColorToken) -> &'static str {
    match token {
        ColorToken::Background => "\x1b[48;2;11;16;32m",
        ColorToken::Surface => "\x1b[48;2;17;24;39m",
        ColorToken::Border => "\x1b[48;2;36;48;68m",
        ColorToken::BorderActive => "\x1b[48;2;56;189;248m",
        ColorToken::Text => "\x1b[48;2;229;231;235m",
        ColorToken::Muted => "\x1b[48;2;148;163;184m",
        ColorToken::Success | ColorToken::Added => "\x1b[48;2;34;197;94m",
        ColorToken::Error | ColorToken::Removed => "\x1b[48;2;239;68;68m",
        ColorToken::Warning | ColorToken::Changed => "\x1b[48;2;234;179;8m",
        ColorToken::Info => "\x1b[48;2;6;182;212m",
        ColorToken::AddedBackground => "\x1b[48;2;5;46;22m",
        ColorToken::RemovedBackground => "\x1b[48;2;69;10;10m",
        ColorToken::HunkBackground => "\x1b[48;2;12;35;64m",
    }
}
