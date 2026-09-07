use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::helpers::{normalize_optional_value, parse_optional_value};
use crate::optionality::{Optional, Required};
use crate::validator::{ConfigValidator, NoValidation};

macro_rules! define_float_type {
    ($name:ident, $ty:ty, $label:expr) => {
        paste::paste! {
            #[doc = concat!("A ", $label, " config value.")]
            pub struct $name<V = NoValidation, O = Required>(PhantomData<(V, O)>);
        }

        impl<V: ConfigValidator<Option<$ty>>> ConfigType for $name<V, Required> {
            type Stored = Option<$ty>;
            type Default = $ty;
            type Value = $ty;

            fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
                stored.unwrap_or(*default)
            }

            fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
                let is_none = input.eq_ignore_ascii_case("none");
                if is_none {
                    return Err(ConfigError::InvalidValue(format!(
                        "'none' is not valid for a required {} field",
                        $label
                    )));
                }
                let parsed: $ty = input.parse().map_err(|_| {
                    ConfigError::InvalidValue(format!("invalid {}: '{}'", $label, input))
                })?;
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

        impl<V: ConfigValidator<Option<$ty>>> ConfigType for $name<V, Optional> {
            type Stored = Option<$ty>;
            type Default = Option<$ty>;
            type Value = Option<$ty>;

            fn get(stored: &Self::Stored, default: &Self::Default) -> Self::Value {
                (*stored).or(*default)
            }

            fn parse(input: &str, default: &Self::Default) -> Result<Self::Stored, ConfigError> {
                let parsed = parse_optional_value(input, |s| {
                    s.parse::<$ty>().map_err(|_| {
                        ConfigError::InvalidValue(format!("invalid {}: '{}'", $label, s))
                    })
                })?;
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
    };
}

define_float_type!(F32, f32, "f32");
define_float_type!(F64, f64, "f64");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_required_default() {
        let default = 1.5f64;
        assert_eq!(F64::<NoValidation, Required>::get(&None, &default), 1.5);
        assert_eq!(
            F64::<NoValidation, Required>::get(&Some(2.25), &default),
            2.25
        );
    }

    #[test]
    fn f64_optional_accepts_none() {
        let default = Some(3.5f64);
        assert_eq!(
            F64::<NoValidation, Optional>::parse("none", &default).unwrap(),
            None
        );
        assert_eq!(
            F64::<NoValidation, Optional>::get(&None, &default),
            Some(3.5)
        );
    }
}
