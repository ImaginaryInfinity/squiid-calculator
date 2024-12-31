use std::sync::LazyLock;

use regex::Regex;

/// Identifier string
pub static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap());

/// Numeric string
pub static NUMERIC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[-]?(?:[0-9]*\.?[0-9]+(?:[eE][-+]?\d+(?:\.\d+)?)?|[0-9]+)$").unwrap()
});
