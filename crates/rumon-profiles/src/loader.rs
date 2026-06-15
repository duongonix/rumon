//! Profile loading from built-in presets and project-local custom files.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{ProfileError, ProfileResult};
use crate::overrides::custom_profile_path_from;
use crate::profile::builtin_profile_content;

/// Loaded profile source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileSource {
    /// Built-in profile bundled with Rumon.
    BuiltIn,
    /// Custom profile loaded from a local `profiles/` file.
    Custom(PathBuf),
}

/// Loaded profile TOML content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadedProfile {
    /// Profile name.
    pub name: String,
    /// TOML content to merge.
    pub content: String,
    /// Where the profile was loaded from.
    pub source: ProfileSource,
}

/// Loads a profile by name.
///
/// Built-in profiles are resolved first. Unknown names are resolved from
/// `profiles/<name>.toml` in the current working directory.
///
/// # Errors
///
/// Returns an error when the profile name is invalid or no profile exists.
pub fn load_profile(name: &str) -> ProfileResult<LoadedProfile> {
    load_profile_from_dir(name, ".")
}

/// Loads a profile by name using a specific base directory for custom profiles.
///
/// # Errors
///
/// Returns an error when the profile name is invalid or no profile exists.
pub fn load_profile_from_dir(
    name: &str,
    base_dir: impl AsRef<Path>,
) -> ProfileResult<LoadedProfile> {
    validate_profile_name(name)?;
    if let Some(content) = builtin_profile_content(name) {
        return Ok(LoadedProfile {
            name: name.to_string(),
            content: content.to_string(),
            source: ProfileSource::BuiltIn,
        });
    }

    let path = custom_profile_path_from(base_dir.as_ref(), name);
    load_custom_profile(name, &path)
}

fn load_custom_profile(name: &str, path: &Path) -> ProfileResult<LoadedProfile> {
    let content = fs::read_to_string(path).map_err(|_| {
        ProfileError::new(format!(
            "profile `{name}` was not found as a built-in profile or at {}",
            path.display()
        ))
    })?;

    Ok(LoadedProfile {
        name: name.to_string(),
        content,
        source: ProfileSource::Custom(path.to_path_buf()),
    })
}

fn validate_profile_name(name: &str) -> ProfileResult<()> {
    if name.trim().is_empty() {
        return Err(ProfileError::new("profile name must not be empty"));
    }
    if name.contains(['/', '\\']) || name.contains("..") {
        return Err(ProfileError::new(format!("invalid profile name: {name}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ProfileSource, load_profile, load_profile_from_dir};
    use std::fs;

    #[test]
    fn loads_builtin_rust_profile() {
        let profile = load_profile("rust").expect("rust profile should load");

        assert_eq!(profile.source, ProfileSource::BuiltIn);
        assert!(profile.content.contains("cargo run"));
    }

    #[test]
    fn rejects_path_like_profile_names() {
        assert!(load_profile("../secret").is_err());
    }

    #[test]
    fn loads_custom_profile_from_base_dir() {
        let base_dir = std::env::temp_dir().join("rumon_custom_profile_test");
        let profiles_dir = base_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).expect("create profiles dir");
        fs::write(
            profiles_dir.join("backend.toml"),
            r#"
            [run]
            cmd = "cargo test"
            "#,
        )
        .expect("write profile");

        let profile =
            load_profile_from_dir("backend", &base_dir).expect("custom profile should load");

        let _ = fs::remove_dir_all(base_dir);

        assert!(matches!(profile.source, ProfileSource::Custom(_)));
        assert!(profile.content.contains("cargo test"));
    }
}
