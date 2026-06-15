//! Plugin manifest metadata.

/// Minimal plugin manifest metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginManifest {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
}

impl PluginManifest {
    /// Creates a plugin manifest.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PluginManifest;

    #[test]
    fn manifest_constructor_sets_fields() {
        let manifest = PluginManifest::new("git", "1.0.0");

        assert_eq!(manifest.name, "git");
        assert_eq!(manifest.version, "1.0.0");
    }
}
