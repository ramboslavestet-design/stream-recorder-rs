//! A generic, trait-based Rust config framework with a declarative macro.
//!
//! Define a typed, validated configuration schema using [`define_config!`],
//! and let the macro generate the `Config` struct, category/key enums,
//! typed getters, and string-keyed access methods.
//!
//! # Quick start
//!
//! ```ignore
//! use apto::{define_config, types::*};
//!
//! define_config! {
//!     @root {
//!         debug: Bool = false, "Enable debug logging",
//!     }
//!     server {
//!         host: Text = "localhost".to_string(), "Server hostname",
//!         port: U32 = 8080, "Listen port",
//!     }
//! }
//!
//! let mut c = Config::default();
//! assert_eq!(c.get_host(), "localhost");
//! c.set_value("host", "example.com").unwrap();
//! assert_eq!(c.get_host(), "example.com");
//! ```
//!
//! # Core concepts
//!
//! - [`ConfigType`] — trait that defines how a value is stored, parsed,
//!   formatted, and validated. Implement it for custom setting types.
//! - [`ConfigValidator`] — trait for per-field validation logic. Attached
//!   to a field via a type parameter (e.g. `U32<MyValidator>`).
//! - [`Required`] / [`Optional`] — markers controlling whether a setting
//!   can be absent. `Text` is required; `Text<Optional>` is optional.
//! - [`ArrayOf<T>`] — wraps any `ConfigType` into an array field.
//!   `ArrayOf<U32>` stores `Vec<u32>`.
//! - [`define_config!`] — the entry-point macro. Parses `@root` and category
//!   blocks, then delegates to the code generator.
//!
//! # Built-in types
//!
//! | Category | Types |
//! |----------|-------|
//! | Strings  | [`Text`] |
//! | Booleans | [`Bool`] |
//! | Integers | [`U8`], [`U16`], [`U32`], [`U64`], [`I8`], [`I16`], [`I32`], [`I64`] |
//! | Floats   | [`F32`], [`F64`] |
//! | Lists    | [`List`] (string list), [`ArrayOf<T>`] (any type), [`TableList<T>`] (array of tables) |
//!
//! # What stays in the application
//!
//! This crate is purely the framework. The application is responsible for:
//! - Invoking [`define_config!`] to create the concrete `Config` struct
//! - Implementing [`ConfigValidator`] for application-specific constraints
//! - Implementing [`ConfigType`] for custom value types (e.g. durations, file sizes)
//! - Persistence (TOML reading/writing, config file path resolution)
//! - Global singleton setup (optional)

pub mod config_table;
pub mod config_type;
pub mod error;
pub mod helpers;
pub mod macros;
pub mod optionality;
pub mod types;
pub mod validator;

// Re-exports
pub use config_table::ConfigTable;
pub use config_type::ConfigType;
pub use error::ConfigError;
pub use macros::{apply_value, validate_field};
pub use optionality::{Optional, Required};
pub use paste;
pub use types::*;
pub use validator::{ConfigValidator, NoValidation};
