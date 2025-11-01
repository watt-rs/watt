/// Imports
use once_cell::sync::Lazy;
use regex::bytes::Regex;

/// Regex
static RX_SNAKE_CASE: Lazy<Regex> = Lazy::new(|| Regex::new("^[a-z0-9_]+$").unwrap());
static RX_CAMEL_CASE: Lazy<Regex> =
    Lazy::new(|| Regex::new("^[a-z1-9]+(?:[A-Z][a-z1-9]+)+$").unwrap());
static RX_PASCAL_CASE: Lazy<Regex> = Lazy::new(|| Regex::new("^(?:[A-Z][a-z1-9]+)+$").unwrap());

/// Checking given string is `snake_case`
pub fn is_snake_case(string: &str) -> bool {
    RX_SNAKE_CASE.is_match(string.as_bytes())
}

/// Checking given string is `camelCase`
pub fn is_camel_case(string: &str) -> bool {
    RX_CAMEL_CASE.is_match(string.as_bytes())
}

/// Checking given string is `PascalCase`
pub fn is_pascal_case(string: &str) -> bool {
    RX_PASCAL_CASE.is_match(string.as_bytes())
}
