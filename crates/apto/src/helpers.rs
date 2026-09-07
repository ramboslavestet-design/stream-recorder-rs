use crate::error::ConfigError;

/// Parse a string as an optional value, treating `"none"` (case-insensitive) as `None`.
pub fn parse_optional_value<T, F>(input: &str, parse_value: F) -> Result<Option<T>, ConfigError>
where
    F: FnOnce(&str) -> Result<T, ConfigError>,
{
    if input.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        parse_value(input).map(Some)
    }
}

/// Collapse an explicit value that matches the default back to `None`.
///
/// This keeps the serialized TOML clean — values equal to the default are
/// omitted rather than stored redundantly.
pub fn normalize_optional_value<T: PartialEq>(parsed: Option<T>, default: Option<T>) -> Option<T> {
    if parsed == default { None } else { parsed }
}

/// Collapse a parsed list back to `None` when it matches the default list.
pub fn normalize_list_value(
    parsed: Option<Vec<String>>,
    default: &[String],
) -> Option<Vec<String>> {
    match parsed.as_ref() {
        Some(values) if values == default => None,
        _ => parsed,
    }
}

/// Parse a comma-separated list from a string.
pub fn parse_csv_list(input: &str) -> Option<Vec<String>> {
    if input.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(
            input
                .split(',')
                .map(|value| value.trim().to_string())
                .collect(),
        )
    }
}
