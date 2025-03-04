#![allow(clippy::unwrap_used)]

use std::sync::LazyLock;

use regex::Regex;

/// Identifier string
pub static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap());
