use std::marker::PhantomData;

use crate::config_type::ConfigType;
use crate::error::ConfigError;
use crate::helpers::{normalize_optional_value, parse_optional_value};
use crate::optionality::{Optional, Required};
use crate::validator::{ConfigValidator, NoValidation};

macro_rules! define_int_type {
    ($name:ident, $ty:ty, $label:expr) => {
        paste::paste! {
            #[doc = concat!("A ", $label, " config value.\n\n# Required (default)\nAlways has a value — falls back to the default when absent.\n\n# Optional (`", stringify!($name), "<V, Optional>`)\nMay be absent — getter returns `Option<", $label, ">`.")]
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
                    s.parse::<$ty>()
                        .map_err(|_| ConfigError::InvalidValue(format!("invalid {}: '{}'", $label, s)))
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

define_int_type!(U8, u8, "u8");
define_int_type!(U16, u16, "u16");
define_int_type!(U32, u32, "u32");
define_int_type!(U64, u64, "u64");
define_int_type!(I8, i8, "i8");
define_int_type!(I16, i16, "i16");
define_int_type!(I32, i32, "i32");
define_int_type!(I64, i64, "i64");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_required_default() {
        let default = 26u32;
        assert_eq!(U32::<NoValidation, Required>::get(&None, &default), 26);
        assert_eq!(U32::<NoValidation, Required>::get(&Some(30), &default), 30);
    }

    #[test]
    fn u32_required_rejects_none() {
        assert!(U32::<NoValidation, Required>::parse("none", &26u32).is_err());
    }

    #[test]
    fn u32_optional_accepts_none() {
        let default = Some(12u32);
        assert_eq!(
            U32::<NoValidation, Optional>::parse("none", &default).unwrap(),
            None
        );
        assert_eq!(
            U32::<NoValidation, Optional>::get(&None, &default),
            Some(12)
        );
    }

    #[test]
    fn u64_large_values() {
        let default = 1_000_000_000_000u64;
        assert_eq!(
            U64::<NoValidation, Required>::get(&None, &default),
            1_000_000_000_000
        );
        assert_eq!(
            U64::<NoValidation, Required>::format(&None, &default),
            "1000000000000"
        );
    }

    #[test]
    fn i32_signed_values() {
        let default = -5i32;
        assert_eq!(I32::<NoValidation, Required>::get(&None, &default), -5);
        assert_eq!(I32::<NoValidation, Required>::get(&Some(10), &default), 10);
    }
}
