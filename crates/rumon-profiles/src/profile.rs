//! Built-in profile metadata.

/// Built-in profile names supported by the roadmap.
pub const BUILT_IN_PROFILES: &[&str] = &["rust", "node", "go", "python", "docker"];

/// Returns whether a profile name is built in.
#[must_use]
pub fn is_builtin_profile(name: &str) -> bool {
    BUILT_IN_PROFILES.contains(&name)
}

/// Returns built-in profile TOML content.
#[must_use]
pub fn builtin_profile_content(name: &str) -> Option<&'static str> {
    match name {
        "rust" => Some(
            r#"
[run]
cmd = "cargo run"

[watch]
paths = ["src"]
extensions = ["rs", "toml"]
"#,
        ),
        "node" => Some(
            r#"
[run]
cmd = "npm run dev"

[watch]
paths = ["src"]
extensions = ["js", "jsx", "ts", "tsx"]
ignore = ["node_modules", "dist"]
"#,
        ),
        "go" => Some(
            r#"
[run]
cmd = "go run ."

[watch]
paths = ["."]
extensions = ["go"]
ignore = ["vendor"]
"#,
        ),
        "python" => Some(
            r#"
[run]
cmd = "python app.py"

[watch]
paths = ["."]
extensions = ["py"]
ignore = ["__pycache__", ".venv"]
"#,
        ),
        "docker" => Some(
            r#"
[run]
cmd = "docker compose up"

[watch]
paths = ["."]
ignore = [".git", "target", "node_modules", "dist"]
"#,
        ),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{builtin_profile_content, is_builtin_profile};

    #[test]
    fn recognizes_rust_profile() {
        assert!(is_builtin_profile("rust"));
        assert!(!is_builtin_profile("unknown"));
    }

    #[test]
    fn returns_node_profile_content() {
        let content = builtin_profile_content("node").expect("node profile should exist");

        assert!(content.contains("npm run dev"));
        assert!(content.contains("tsx"));
    }
}
