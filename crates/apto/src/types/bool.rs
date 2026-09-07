use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::helpers::{normalize_optional_value, parse_optional_value};
use crate::optionality::{Optional, Required};
use crate::validator::{ConfigValidator, NoValidation};

/// A boolean config value.
///
/// Accepts `"true"`, `"false"`, `"yes"`, `"no"`, `"1"`, `"0"` on parse.
pub struct Bool<V = NoValidation, O = Required>(PhantomData<(V, O)>);

fn parse_bool(input: &str) -> Result<bool, ConfigError> {
    match input.trim().to_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => Ok(true),
        "false" | "no" | "0" | "off" => Ok(false),
        other => Err(ConfigError::InvalidValue(format!(
            "invalid boolean: '{other}' (expected true/false/yes/no/1/0)"
        ))),
    }
}

impl<V: ConfigValidator<Option<bool>>> ConfigType for Bool<V, Required> {
    type Stored = Option<bool>;
    type Default = bool;
    type Value = bool;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.unwrap_or(*default)
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        let is_none = input.eq_ignore_ascii_case("none");
        if is_none {
            return Err(ConfigError::InvalidValue(
                "'none' is not valid for a required boolean field".into(),
            ));
        }
        let parsed = parse_bool(input)?;
        Ok(normalize_optional_value(Some(parsed), Some(*default)))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        stored.unwrap_or(*default).to_string()
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        Some(default)
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

impl<V: ConfigValidator<Option<bool>>> ConfigType for Bool<V, Optional> {
    type Stored = Option<bool>;
    type Default = Option<bool>;
    type Value = Option<bool>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        (*stored).or(*default)
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        let parsed = parse_optional_value(input, parse_bool)?;
        Ok(normalize_optional_value(parsed, *default))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        stored
            .or(*default)
            .map(|v| v.to_string())
            .unwrap_or_else(|| "none".to_string())
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
    fn required_bool_default() {
        let default = true;
        assert!(Bool::<NoValidation, Required>::get(&None, &default));
        assert!(!Bool::<NoValidation, Required>::get(&Some(false), &default));
    }

    #[test]
    fn bool_parse_variants() {
        let default = false;
        assert_eq!(
            Bool::<NoValidation, Required>::parse("true", &default).unwrap(),
            Some(true)
        );
        assert_eq!(
            Bool::<NoValidation, Required>::parse("yes", &default).unwrap(),
            Some(true)
        );
        assert_eq!(
            Bool::<NoValidation, Required>::parse("1", &default).unwrap(),
            Some(true)
        );
        assert_eq!(
            Bool::<NoValidation, Required>::parse("on", &default).unwrap(),
            Some(true)
        );
        assert_eq!(
            Bool::<NoValidation, Required>::parse("false", &default).unwrap(),
            None
        );
    }

    #[test]
    fn bool_rejects_invalid() {
        let default = false;
        assert!(Bool::<NoValidation, Required>::parse("maybe", &default).is_err());
    }
}
