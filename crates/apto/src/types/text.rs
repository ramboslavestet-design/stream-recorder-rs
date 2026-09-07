use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::helpers::{normalize_optional_value, parse_optional_value};
use crate::optionality::{Optional, Required};
use crate::validator::{ConfigValidator, NoValidation};

/// A text (string) config value.
///
/// # Required (default)
/// Always has a value — falls back to the default string when absent.
///
/// # Optional (`Text<V, Optional>`)
/// May be absent — getter returns `Option<String>`.
pub struct Text<V = NoValidation, O = Required>(PhantomData<(V, O)>);

impl<V: ConfigValidator<Option<String>>> ConfigType for Text<V, Required> {
    type Stored = Option<String>;
    type Default = String;
    type Value = String;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.clone().unwrap_or_else(|| default.clone())
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        let is_none = input.eq_ignore_ascii_case("none");
        if is_none {
            return Err(ConfigError::InvalidValue(
                "'none' is not valid for a required text field".into(),
            ));
        }
        let parsed = Some(input.to_string());
        Ok(normalize_optional_value(parsed, Some(default.clone())))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        stored.clone().unwrap_or_else(|| default.clone())
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        Some(default)
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

impl<V: ConfigValidator<Option<String>>> ConfigType for Text<V, Optional> {
    type Stored = Option<String>;
    type Default = Option<String>;
    type Value = Option<String>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.clone().or_else(|| default.clone())
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        let parsed = parse_optional_value(input, |s| Ok(s.to_string()))?;
        Ok(normalize_optional_value(parsed, default.clone()))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        stored
            .clone()
            .or_else(|| default.clone())
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
    fn required_text_uses_default_when_absent() {
        let default = "hello".to_string();
        assert_eq!(
            Text::<NoValidation, Required>::get(&None, &default),
            "hello"
        );
        assert_eq!(
            Text::<NoValidation, Required>::get(&Some("world".to_string()), &default),
            "world"
        );
    }

    #[test]
    fn required_text_rejects_none() {
        let default = "hello".to_string();
        assert!(Text::<NoValidation, Required>::parse("none", &default).is_err());
    }

    #[test]
    fn optional_text_accepts_none() {
        let default = Some("hello".to_string());
        assert_eq!(
            Text::<NoValidation, Optional>::parse("none", &default).unwrap(),
            None
        );
        assert_eq!(
            Text::<NoValidation, Optional>::get(&None, &default),
            Some("hello".to_string())
        );
    }
}
