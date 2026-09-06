use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::validator::{ConfigValidator, NoValidation};

/// Wrap any [`ConfigType`] into an array (vector) version.
///
/// `ArrayOf<U32>` stores `Option<Vec<u32>>` and returns `Vec<u32>`.
pub struct ArrayOf<T, V = NoValidation>(PhantomData<(T, V)>);

impl<T: ConfigType, V: ConfigValidator<Option<Vec<T::Stored>>>> ConfigType for ArrayOf<T, V> {
    type Stored = Option<Vec<T::Stored>>;
    type Default = Vec<T::Default>;
    type Value = Vec<T::Value>;

    fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
        match stored {
            Some(v) => v
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let d = default.get(i).cloned().unwrap_or_default();
                    T::get(s, &d)
                })
                .collect(),
            None => default
                .iter()
                .map(|d| {
                    let s = T::stored_from_default(d.clone());
                    T::get(&s, d)
                })
                .collect(),
        }
    }

    fn parse(input: &str, _default: &Self::Default) -> Result<Self::Stored, ConfigError> {
        if input.eq_ignore_ascii_case("none") {
            return Ok(None);
        }
        let items: Vec<T::Stored> = input
            .split(',')
            .map(|s| {
                let trimmed = s.trim();
                T::parse(trimmed, &T::Default::default())
                    .map_err(|e| ConfigError::InvalidValue(format!("invalid '{}': {}", trimmed, e)))
            })
            .collect::<Result<_, _>>()?;
        Ok(Some(items))
    }

    fn format(stored: &Self::Stored, default: &Self::Default) -> String {
        let fallback_default = T::Default::default();
        let items: Vec<String> = match stored {
            Some(v) => v
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let d = default.get(i).unwrap_or(&fallback_default);
                    T::format(item, d)
                })
                .collect(),
            None if default.is_empty() => return "none".to_string(),
            None => default
                .iter()
                .map(|d| {
                    let s = T::stored_from_default(d.clone());
                    T::format(&s, d)
                })
                .collect(),
        };
        if items.is_empty() {
            "none".to_string()
        } else {
            items.join(", ")
        }
    }

    fn stored_from_default(default: Self::Default) -> Self::Stored {
        if default.is_empty() {
            None
        } else {
            Some(
                default
                    .into_iter()
                    .map(|d| T::stored_from_default(d))
                    .collect(),
            )
        }
    }

    fn validate(stored: &Self::Stored) -> Result<(), ConfigError> {
        V::validate(stored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Bool, Text, U32};

    #[test]
    fn array_of_u32_parses_csv() {
        let default = Vec::<u32>::new();
        let parsed = ArrayOf::<U32>::parse("1, 2, 3", &default)
            .expect("valid CSV should parse")
            .expect("should not be None");
        assert_eq!(parsed, vec![Some(1u32), Some(2u32), Some(3u32)]);
    }

    #[test]
    fn array_of_u32_gets_default_when_absent() {
        let default = vec![42u32];
        let val = ArrayOf::<U32>::get(&None, &default);
        assert_eq!(val, vec![42u32]);
    }

    #[test]
    fn array_of_text_parses_csv() {
        let default = Vec::<String>::new();
        let parsed = ArrayOf::<Text>::parse("hello, world", &default)
            .expect("valid CSV should parse")
            .expect("should not be None");
        assert_eq!(
            parsed,
            vec![Some("hello".to_string()), Some("world".to_string())]
        );
    }

    #[test]
    fn array_of_accepts_none() {
        let default = vec!["a".to_string()];
        let result = ArrayOf::<Text>::parse("none", &default).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn array_of_bool_parses() {
        let default = Vec::new();
        let parsed = ArrayOf::<Bool>::parse("true, false", &default)
            .expect("valid CSV should parse")
            .expect("should not be None");
        assert_eq!(parsed, vec![Some(true), None]);
    }

    #[test]
    fn array_of_u32_round_trips_format() {
        let default = vec![0u32];
        let stored = Some(vec![Some(1u32), Some(2u32), Some(3u32)]);
        let formatted = ArrayOf::<U32>::format(&stored, &default);
        assert_eq!(formatted, "1, 2, 3");
    }

    #[test]
    fn array_of_u32_round_trips_get() {
        let default = vec![0u32];
        let stored = Some(vec![Some(1u32), Some(2u32)]);
        let val = ArrayOf::<U32>::get(&stored, &default);
        assert_eq!(val, vec![1u32, 2u32]);
    }
}
