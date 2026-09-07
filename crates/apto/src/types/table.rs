use std::marker::PhantomData;

use crate::config_table::ConfigTable;
use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::validator::{ConfigValidator, NoValidation};

/// A TOML array-of-tables (`[[key]]`) config value.
///
/// Each element is a user-defined struct that implements [`ConfigTable`].
///
/// String input accepts JSON arrays (e.g. `[{"field":"value"}]`) or `"none"`.
pub struct TableList<T: ConfigTable, V = NoValidation>(PhantomData<(T, V)>);

impl<T: ConfigTable, V: ConfigValidator<Option<Vec<T>>>> ConfigType for TableList<T, V> {
    type Stored = Option<Vec<T>>;
    type Default = Vec<T>;
    type Value = Vec<T>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        stored.clone().unwrap_or_else(|| default.clone())
    }

    fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        if input.eq_ignore_ascii_case("none") {
            return Ok(None);
        }
        let parsed: Vec<T> = serde_json::from_str(input)
            .map_err(|e| ConfigError::InvalidValue(format!("invalid table list JSON: {e}")))?;
        if parsed == *default {
            Ok(None)
        } else {
            Ok(Some(parsed))
        }
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        let values = stored.as_ref().unwrap_or(default);
        serde_json::to_string(values).unwrap_or_else(|_| "none".to_string())
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        Some(default)
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct Entry {
        name: String,
        value: Option<u32>,
    }

    #[test]
    fn table_list_parse_json() {
        let default = Vec::<Entry>::new();
        let json = r#"[{"name":"foo","value":42}]"#;
        let parsed = TableList::<Entry, NoValidation>::parse(json, &default)
            .expect("valid JSON should parse")
            .expect("should not be None");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].name, "foo");
        assert_eq!(parsed[0].value, Some(42));
    }

    #[test]
    fn table_list_none_clears() {
        let default = vec![Entry {
            name: "default".to_string(),
            value: None,
        }];
        assert_eq!(
            TableList::<Entry, NoValidation>::parse("none", &default).unwrap(),
            None
        );
    }
}
