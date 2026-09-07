use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Behavior shared by all config value types.
///
/// A `ConfigType` owns the conversion rules for a setting:
/// how it is stored in TOML, parsed from a string, displayed,
/// and what the typed getter returns.
pub trait ConfigType {
    /// The raw serialized representation stored in the `Config` struct.
    type Stored: Clone + Default + Serialize + for<'de> Deserialize<'de>;

    /// The effective default value used when the setting is absent.
    type Default: Clone + Default;

    /// The value returned by the typed getter.
    type Value: Clone;

    /// Convert the stored value and default into the typed getter output.
    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value;

    /// Parse a string into the stored representation.
    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError>;

    /// Format the stored value for display.
    fn format(stored: &Self::Stored, default: &Self::Default) -> String;

    /// Format the effective default for display.
    fn format_default(default: &Self::Default) -> String {
        let default_stored = Self::stored_from_default(default.clone());
        Self::format(&default_stored, default)
    }

    /// Build a stored value from a default value.
    fn stored_from_default(default: Self::Default) -> Self::Stored;

    /// Validate the stored representation.
    fn validate(stored: &Self::Stored) -> Result<(), ConfigError>;

    /// Reset a field back to its default (erase user-set value).
    fn reset_value(default: &Self::Default) -> Self::Stored {
        let _ = default;
        Self::Stored::default()
    }
}
