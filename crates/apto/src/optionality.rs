/// Marker for a required (non-optional) config value.
///
/// Required fields always produce a value — either from user config or the default.
/// The typed getter returns `T`, not `Option<T>`, and `"none"` is rejected on parse.
pub struct Required;

/// Marker for an optional config value.
///
/// Optional fields may be absent. The typed getter returns `Option<T>`,
/// and `"none"` is accepted on parse (clears the value).
pub struct Optional;
