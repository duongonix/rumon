//! Reusable profile boundary for Rumon.

mod error;
mod loader;
mod overrides;
mod profile;
mod selector;

pub use error::{ProfileError, ProfileResult};
pub use loader::{LoadedProfile, ProfileSource, load_profile, load_profile_from_dir};
pub use overrides::{custom_profile_path, custom_profile_path_from};
pub use profile::{BUILT_IN_PROFILES, builtin_profile_content, is_builtin_profile};
pub use selector::extract_profile_name;
