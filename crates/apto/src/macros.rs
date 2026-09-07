/// Declaratively define a configuration schema.
///
/// Invoking this macro generates:
/// - A `Config` struct with one field per setting
/// - A `ConfigCategory` enum with one variant per category block
/// - A `ConfigKey` enum with one variant per setting
/// - Typed getter methods (`get_<field>`) on `Config`
/// - String-keyed access methods (`get_value`, `set_value`, `reset_key`,
///   `get_default_string`, `get_description`, `validate`)
///
/// # Syntax
///
/// ```ignore
/// define_config! {
///     @root {
///         root_field: Type = default, "Description",
///     }
///     category_name {
///         field: Type<Validator, Optionality> = default, "Description",
///     }
/// }
/// ```
///
/// ## Root fields (`@root` block)
///
/// Fields declared in the `@root` block become top-level TOML keys with no
/// section wrapper. They have `category() -> None`.
///
/// ## Category blocks
///
/// Each `ident { ... }` block becomes a `[ident]` section in the TOML file.
/// Fields inside have `category() -> Some(ConfigCategory::ident)`.
///
/// ## Field format
///
/// Each field follows: `name: Type = default_expr, "description text",`
///
/// - **`name`** — snake_case identifier; used to generate the `get_<name>()` getter
/// - **`Type`** — a type implementing [`ConfigType`](crate::ConfigType), such as
///   [`Text`](crate::types::Text), [`U32`](crate::types::U32),
///   [`Bool`](crate::types::Bool), [`ArrayOf<T>`](crate::types::ArrayOf),
///   [`List`](crate::types::List), or a custom type
/// - **`default`** — a Rust expression for the effective default value
///   (e.g. `42`, `"hello".to_string()`, `Vec::new()`, `None`)
/// - **`"description"`** — a string literal describing the setting
///
/// Optionality is encoded via the type parameter:
/// - `Text` = required, `Text<Optional>` = optional
/// - `U32` = required, `U32<Optional>` = optional
/// - `Bool` = required, `Bool<Optional>` = optional
///
/// Validation is encoded via the first type parameter:
/// - `U32<MyValidator>` = required with validation
/// - `U32<MyValidator, Optional>` = optional with validation
///
/// # Generated types
///
/// | Type | Purpose |
/// |------|---------|
/// | `Config` | Struct with one `Option<T::Stored>` field per setting |
/// | `ConfigCategory` | Enum listing all category blocks (`@root` excluded) |
/// | `ConfigKey` | Enum listing all settings (root + categorized) |
///
/// # Generated methods on `Config`
///
/// | Method | Returns | Description |
/// |--------|---------|-------------|
/// | `get_<field>()` | typed value | Access the effective value (falls back to default) |
/// | `validate()` | `Result<()>` | Run per-field validators |
/// | `get_value(key)` | `String` | Format a setting's current value |
/// | `set_value(key, val)` | `Result<()>` | Parse a string and set a field |
/// | `reset_key(key)` | `Result<String>` | Reset a field to its default |
/// | `get_default_string(key)` | `String` | Format a setting's default |
/// | `get_description(key)` | `String` | Get a setting's description |
///
/// # Generated methods on `ConfigKey`
///
/// | Method | Returns | Description |
/// |--------|---------|-------------|
/// | `as_str()` | `&str` | The setting's field name |
/// | `from_key(s)` | `Option<Self>` | Look up a key by name |
/// | `category()` | `Option<ConfigCategory>` | `None` for root fields |
/// | `all()` | `&[Self]` | All known keys |
///
/// # Example
///
/// ```ignore
/// use apto::{define_config, types::*};
///
/// define_config! {
///     @root {
///         debug: Bool = false, "Enable debug logging",
///     }
///     server {
///         host: Text = "localhost".to_string(), "Server hostname",
///         port: U32 = 8080, "Listen port",
///         tags: ArrayOf<Text> = Vec::new(), "Search tags",
///     }
/// }
///
/// let c = Config::default();
/// assert_eq!(c.get_host(), "localhost");
/// assert_eq!(c.get_port(), 8080);
/// assert!(!c.get_debug());
/// assert!(ConfigKey::debug.category().is_none());
/// ```
#[macro_export]
macro_rules! define_config {
    // Entry with `@root { }` block
    (
        @root {
            $($root_field:ident : $root_type:ty = $root_default:expr , $root_desc:expr),* $(,)?
        }
        $($category:ident {
            $($field:ident : $type:ty = $default:expr , $desc:expr),* $(,)?
        })*
    ) => {
        $crate::gen_config! {
            roots: [$($root_field: $root_type = $root_default, $root_desc,)*]
            cats: [$($category [$($field: $type = $default, $desc,)*])*]
        }
    };

    // Entry without `root` block
    (
        $($category:ident {
            $($field:ident : $type:ty = $default:expr , $desc:expr),* $(,)?
        })*
    ) => {
        $crate::gen_config! {
            roots: []
            cats: [$($category [$($field: $type = $default, $desc,)*])*]
        }
    };
}

/// Internal macro that generates the `Config` struct, enums, and methods.
///
/// This is called by [`define_config!`] and should not be invoked directly.
#[macro_export]
macro_rules! gen_config {
    (
        roots: [$($root_field:ident : $root_type:ty = $root_default:expr , $root_desc:expr,)*]
        cats: [$($category:ident [$($field:ident : $type:ty = $default:expr , $desc:expr,)*])*]
    ) => {
        // Config struct
        #[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Default)]
        #[serde(default)]
        pub struct Config {
            $(
                #[serde(default, skip_serializing_if = "Option::is_none")]
                pub $root_field: <$root_type as $crate::ConfigType>::Stored,
            )*
            $(
                $(
                    #[serde(default, skip_serializing_if = "Option::is_none")]
                    pub $field: <$type as $crate::ConfigType>::Stored,
                )*
            )*
        }

        // ConfigCategory enum (no root variant)
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum ConfigCategory {
            $(
                $category,
            )*
        }

        impl ConfigCategory {
            pub fn as_str(&self) -> &str {
                match self {
                    $(ConfigCategory::$category => stringify!($category),)*
                }
            }

            pub const fn all() -> &'static [Self] {
                &[$(ConfigCategory::$category,)*]
            }

            pub fn keys(&self) -> &'static [ConfigKey] {
                match self {
                    $(
                        ConfigCategory::$category => &[
                            $(ConfigKey::$field,)*
                        ],
                    )*
                }
            }

            pub fn display_name(&self) -> String {
                let s = self.as_str();
                s.split('_')
                    .filter(|seg| !seg.is_empty())
                    .map(|seg| {
                        let mut chars = seg.chars();
                        match chars.next() {
                            Some(first) => format!(
                                "{}{}",
                                first.to_ascii_uppercase(),
                                chars.as_str()
                            ),
                            None => String::new(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }

        // ConfigKey enum (all fields, root + categorized)
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        #[allow(non_camel_case_types)]
        pub enum ConfigKey {
            $(
                $root_field,
            )*
            $(
                $(
                    $field,
                )*
            )*
        }

        impl ConfigKey {
            pub fn as_str(&self) -> &str {
                match self {
                    $(ConfigKey::$root_field => stringify!($root_field),)*
                    $(
                        $(ConfigKey::$field => stringify!($field),)*
                    )*
                }
            }

            pub fn from_key(s: &str) -> Option<Self> {
                match s {
                    $(stringify!($root_field) => Some(ConfigKey::$root_field),)*
                    $(
                        $(stringify!($field) => Some(ConfigKey::$field),)*
                    )*
                    _ => None,
                }
            }

            pub const fn all() -> &'static [Self] {
                &[
                    $(ConfigKey::$root_field,)*
                    $(
                        $(ConfigKey::$field,)*
                    )*
                ]
            }

            pub fn category(&self) -> Option<ConfigCategory> {
                match self {
                    $(ConfigKey::$root_field => None,)*
                    $(
                        $(ConfigKey::$field => Some(ConfigCategory::$category),)*
                    )*
                }
            }

        }

        // Typed getters
        impl Config {
            $(
                $crate::paste::paste! {
                    pub fn [<get_ $root_field>](&self) -> <$root_type as $crate::ConfigType>::Value {
                        <$root_type as $crate::ConfigType>::get(
                            &self.$root_field,
                            &($root_default),
                        )
                    }
                }
            )*
            $(
                $(
                    $crate::paste::paste! {
                        pub fn [<get_ $field>](&self) -> <$type as $crate::ConfigType>::Value {
                            <$type as $crate::ConfigType>::get(
                                &self.$field,
                                &($default),
                            )
                        }
                    }
                )*
            )*
        }

        // String-keyed access methods
        impl Config {
            pub fn validate(&self) -> std::result::Result<(), $crate::ConfigError> {
                $(
                    $crate::validate_field::<$root_type>(
                        stringify!($root_field),
                        &self.$root_field,
                    )?;
                )*
                $(
                    $(
                        $crate::validate_field::<$type>(
                            stringify!($field),
                            &self.$field,
                        )?;
                    )*
                )*
                Ok(())
            }

            pub fn get_value(&self, key: &str) -> String {
                match ConfigKey::from_key(key) {
                    $(
                        Some(ConfigKey::$root_field) => {
                            <$root_type as $crate::ConfigType>::format(
                                &self.$root_field,
                                &($root_default),
                            )
                        }
                    )*
                    $(
                        $(
                            Some(ConfigKey::$field) => {
                                <$type as $crate::ConfigType>::format(
                                    &self.$field,
                                    &($default),
                                )
                            }
                        )*
                    )*
                    None => "unknown key".to_string(),
                }
            }

            pub fn set_value(&mut self, key: &str, value: &str) -> std::result::Result<(), $crate::ConfigError> {
                match ConfigKey::from_key(key) {
                    $(
                        Some(ConfigKey::$root_field) => {
                            $crate::apply_value::<$root_type>(
                                stringify!($root_field),
                                &mut self.$root_field,
                                value,
                                &($root_default),
                            )?;
                        }
                    )*
                    $(
                        $(
                            Some(ConfigKey::$field) => {
                                $crate::apply_value::<$type>(
                                    stringify!($field),
                                    &mut self.$field,
                                    value,
                                    &($default),
                                )?;
                            }
                        )*
                    )*
                    None => return Err($crate::ConfigError::UnknownKey(key.to_string())),
                }
                Ok(())
            }

            pub fn reset_key(&mut self, key: &str) -> std::result::Result<String, $crate::ConfigError> {
                match ConfigKey::from_key(key) {
                    $(
                        Some(ConfigKey::$root_field) => {
                            self.$root_field = <$root_type as $crate::ConfigType>::reset_value(
                                &($root_default),
                            );
                            return Ok(<$root_type as $crate::ConfigType>::format_default(
                                &($root_default),
                            ));
                        }
                    )*
                    $(
                        $(
                            Some(ConfigKey::$field) => {
                                self.$field = <$type as $crate::ConfigType>::reset_value(
                                    &($default),
                                );
                                return Ok(<$type as $crate::ConfigType>::format_default(
                                    &($default),
                                ));
                            }
                        )*
                    )*
                    None => return Err($crate::ConfigError::UnknownKey(key.to_string())),
                }
            }

            pub fn get_default_string(&self, key: ConfigKey) -> String {
                match key {
                    $(ConfigKey::$root_field => {
                        <$root_type as $crate::ConfigType>::format_default(&($root_default))
                    })*
                    $(
                        $(ConfigKey::$field => {
                            <$type as $crate::ConfigType>::format_default(&($default))
                        })*
                    )*
                }
            }

            pub fn get_description(&self, key: &str) -> String {
                match ConfigKey::from_key(key) {
                    $(Some(ConfigKey::$root_field) => format!($root_desc),)*
                    $(
                        $(Some(ConfigKey::$field) => format!($desc),)*
                    )*
                    None => "unknown key".to_string(),
                }
            }
        }
    };
}

#[doc(hidden)]
pub fn validate_field<T: crate::ConfigType>(
    key: &str,
    value: &T::Stored,
) -> Result<(), crate::error::ConfigError> {
    T::validate(value).map_err(|e| {
        crate::error::ConfigError::InvalidValue(format!("Invalid value for '{}': {}", key, e))
    })
}

#[doc(hidden)]
pub fn apply_value<T: crate::ConfigType>(
    key: &str,
    field: &mut T::Stored,
    raw_value: &str,
    value_default: &T::Default,
) -> Result<(), crate::error::ConfigError> {
    let parsed = T::parse(raw_value, value_default).map_err(|e| {
        crate::error::ConfigError::InvalidValue(format!("Invalid value for '{}': {}", key, e))
    })?;
    validate_field::<T>(key, &parsed)?;
    *field = parsed;
    Ok(())
}
