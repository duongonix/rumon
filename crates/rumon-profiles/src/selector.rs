//! Profile selection helpers.

/// Extracts the top-level `profile = "name"` value from a TOML-like config subset.
#[must_use]
pub fn extract_profile_name(content: &str) -> Option<String> {
    let mut in_top_level = true;
    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_top_level = false;
            continue;
        }
        if !in_top_level {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == "profile" {
            return Some(value.trim().trim_matches('"').to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::extract_profile_name;

    #[test]
    fn extracts_top_level_profile() {
        let content = r#"
        profile = "rust"

        [run]
        cmd = "cargo test"
        "#;

        assert_eq!(extract_profile_name(content), Some("rust".to_string()));
    }
}
