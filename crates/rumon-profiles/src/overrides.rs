//! Custom profile path helpers.

use std::path::PathBuf;

/// Returns the project-local path for a custom profile name.
#[must_use]
pub fn custom_profile_path(name: &str) -> PathBuf {
    custom_profile_path_from(PathBuf::from("."), name)
}

/// Returns the custom profile path from a base directory.
#[must_use]
pub fn custom_profile_path_from(base_dir: impl Into<PathBuf>, name: &str) -> PathBuf {
    base_dir
        .into()
        .join("profiles")
        .join(format!("{name}.toml"))
}

#[cfg(test)]
mod tests {
    use super::{custom_profile_path, custom_profile_path_from};
    use std::path::PathBuf;

    #[test]
    fn builds_custom_profile_path() {
        assert_eq!(
            custom_profile_path("backend"),
            PathBuf::from(".").join("profiles").join("backend.toml")
        );
    }

    #[test]
    fn builds_custom_profile_path_from_base_dir() {
        assert_eq!(
            custom_profile_path_from("workspace", "backend"),
            PathBuf::from("workspace")
                .join("profiles")
                .join("backend.toml")
        );
    }
}
