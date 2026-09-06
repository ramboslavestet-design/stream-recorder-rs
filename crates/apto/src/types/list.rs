use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::helpers::{normalize_list_value, parse_csv_list};
use crate::optionality::{Optional, Required};
use crate::validator::{ConfigValidator, NoValidation};

/// A comma-separated string list config value.
///
/// Serializes as a TOML array of strings (`key = ["a", "b"]`).
pub struct List<V = NoValidation, O = Required>(PhantomData<(V, O)>);

impl<V: ConfigValidator<Option<Vec<String>>>> ConfigType for List<V, Required> {
    type Stored = Option<Vec<String>>;
    type Default = Vec<String>;
    type Value = Vec<String>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.clone().unwrap_or_else(|| default.clone())
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        Ok(normalize_list_value(parse_csv_list(input), default))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        let values = stored.as_ref().unwrap_or(default);
        if values.is_empty() {
            "none".to_string()
        } else {
            values.join(", ")
        }
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        Some(default)
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

impl<V: ConfigValidator<Option<Vec<String>>>> ConfigType for List<V, Optional> {
    type Stored = Option<Vec<String>>;
    type Default = Option<Vec<String>>;
    type Value = Option<Vec<String>>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.clone().or_else(|| default.clone())
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        let parsed = parse_csv_list(input);
        let normalized = match (&parsed, default) {
            (Some(values), Some(default_values)) if values == default_values => None,
            _ => parsed,
        };
        Ok(normalized)
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        let values = stored.as_ref().or(default.as_ref());
        match values {
            Some(values) if values.is_empty() => "none".to_string(),
            Some(values) => values.join(", "),
            None => "none".to_string(),
        }
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        default
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_list_default() {
        let default = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            List::<NoValidation, Required>::get(&None, &default),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn list_parse_csv() {
        let default = Vec::<String>::new();
        assert_eq!(
            List::<NoValidation, Required>::parse("one, two", &default).unwrap(),
            Some(vec!["one".to_string(), "two".to_string()])
        );
    }

    #[test]
    fn list_normalizes_default() {
        let default = vec!["a".to_string(), "b".to_string()];
        assert_eq!(
            List::<NoValidation, Required>::parse("a, b", &default).unwrap(),
            None
        );
    }
}
