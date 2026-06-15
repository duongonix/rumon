//! Text diff entry points and text detection.

use std::path::Path;

use crate::line_diff::{LineChange, diff_lines};
use crate::preview::DiffPreview;

/// Text diff output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextDiff {
    /// Line changes.
    pub changes: Vec<LineChange>,
    /// Preview suitable for UI rendering.
    pub preview: DiffPreview,
}

/// Returns whether a path is likely to be a text file.
#[must_use]
pub fn is_text_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_text_extension)
}

/// Computes a text diff with a preview limit.
#[must_use]
pub fn diff_text(before: &str, after: &str, max_preview_lines: usize) -> TextDiff {
    let changes = diff_lines(before, after);
    let changed_lines: Vec<LineChange> = changes
        .iter()
        .filter(|change| change.is_changed())
        .cloned()
        .collect();
    let preview = DiffPreview::from_changes(&changed_lines, max_preview_lines);
    TextDiff { changes, preview }
}

fn is_text_extension(extension: &str) -> bool {
    matches!(
        extension,
        "rs" | "go"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "py"
            | "json"
            | "yaml"
            | "yml"
            | "toml"
            | "md"
            | "txt"
            | "css"
            | "html"
            | "xml"
            | "sh"
            | "ps1"
    )
}

#[cfg(test)]
mod tests {
    use super::{diff_text, is_text_path};
    use std::path::Path;

    #[test]
    fn detects_text_extensions() {
        assert!(is_text_path(Path::new("src/main.rs")));
        assert!(!is_text_path(Path::new("image.png")));
    }

    #[test]
    fn text_diff_respects_preview_limit() {
        let diff = diff_text("a\nb\nc", "x\ny\nz", 1);

        assert!(diff.preview.truncated);
        assert_eq!(diff.preview.lines.len(), 2);
    }

    #[test]
    fn if_else_insert_preview_has_clean_rows() {
        let before = r#"fn main() {
  let a = 2;
  let b = 1;
  if a > b {
    println!("a is greater than b");
  }
}"#;
        let after = r#"fn main() {
  let a = 2;
  let b = 1;
  if a > b {
    println!("a is greater than b");
  } else {
    println!("a is not greater than b");
  }
}"#;

        let diff = diff_text(before, after, 8);

        assert!(!diff.preview.lines.iter().any(|line| line.contains('\n')));
        assert!(diff.preview.lines.iter().any(|line| line == "+   } else {"));
        assert!(
            diff.preview
                .lines
                .iter()
                .any(|line| line.contains("a is not greater than b"))
        );
    }
}
