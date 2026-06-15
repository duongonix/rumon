//! Terminal layout calculations.

/// A terminal rectangle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    /// X coordinate.
    pub x: u16,
    /// Y coordinate.
    pub y: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

/// Main TUI layout regions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Layout {
    /// Header area.
    pub header: Rect,
    /// Changes panel area.
    pub changes: Rect,
    /// Logs panel area.
    pub logs: Rect,
    /// Footer area.
    pub footer: Rect,
    /// Whether the terminal is too small for the full layout.
    pub too_small: bool,
}

/// Splits terminal dimensions into Rumon TUI regions.
#[must_use]
pub fn split_layout(width: u16, height: u16, left_panel_width: u16) -> Layout {
    if width < 40 || height < 10 {
        let rect = Rect {
            x: 0,
            y: 0,
            width,
            height,
        };
        return Layout {
            header: rect,
            changes: rect,
            logs: rect,
            footer: rect,
            too_small: true,
        };
    }

    let header = Rect {
        x: 0,
        y: 0,
        width,
        height: 2,
    };
    let footer = Rect {
        x: 0,
        y: height.saturating_sub(2),
        width,
        height: 2,
    };
    let body_y = header.height;
    let body_height = height.saturating_sub(header.height + footer.height);

    if width < 100 {
        let half = body_height / 2;
        return Layout {
            header,
            changes: Rect {
                x: 0,
                y: body_y,
                width,
                height: half,
            },
            logs: Rect {
                x: 0,
                y: body_y + half,
                width,
                height: body_height.saturating_sub(half),
            },
            footer,
            too_small: false,
        };
    }

    let left = width.saturating_mul(left_panel_width).saturating_div(100);
    Layout {
        header,
        changes: Rect {
            x: 0,
            y: body_y,
            width: left,
            height: body_height,
        },
        logs: Rect {
            x: left,
            y: body_y,
            width: width.saturating_sub(left),
            height: body_height,
        },
        footer,
        too_small: false,
    }
}

#[cfg(test)]
mod tests {
    use super::split_layout;

    #[test]
    fn wide_layout_splits_horizontally() {
        let layout = split_layout(120, 40, 58);

        assert_eq!(layout.changes.width, 69);
        assert_eq!(layout.logs.width, 51);
        assert!(!layout.too_small);
    }
}
