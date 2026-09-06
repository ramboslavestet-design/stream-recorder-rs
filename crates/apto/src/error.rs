use std::fmt;

/// Errors that can occur during config parsing, validation, and key access.
#[derive(Debug)]
pub enum ConfigError {
    /// A value could not be parsed or validated.
    InvalidValue(String),
    /// A config key was not recognised.
    UnknownKey(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidValue(msg) => write!(f, "{msg}"),
            ConfigError::UnknownKey(key) => write!(f, "unknown key: {key}"),
        }
    }
}

impl std::error::Error for ConfigError {}
