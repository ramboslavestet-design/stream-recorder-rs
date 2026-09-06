use crate::error::ConfigError;

/// A reusable validator for a config value.
///
/// Implement this for marker types that enforce constraints beyond the base
/// parsing behavior of a [`ConfigType`](crate::ConfigType).
pub trait ConfigValidator<T> {
    fn validate(value: &T) -> Result<(), ConfigError>;
}

/// Marker type for config values that do not need extra validation.
pub struct NoValidation;

impl<T> ConfigValidator<T> for NoValidation {
    fn validate(_: &T) -> Result<(), ConfigError> {
        Ok(())
    }
}
