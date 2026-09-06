use serde::Serialize;
use serde::de::DeserializeOwned;

/// A struct that can be used as an element in a `TableList`.
///
/// This is a marker trait — implement it on any struct that should be
/// stored as a TOML array of tables (`[[key]]`).
pub trait ConfigTable: DeserializeOwned + Serialize + Clone + Default + PartialEq {}

impl<T> ConfigTable for T where T: DeserializeOwned + Serialize + Clone + Default + PartialEq {}
