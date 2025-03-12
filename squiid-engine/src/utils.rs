//! - `utils.rs` contains standard utilities that might be used throughout the calculator, such as
//!   [`ID_REGEX`] for identifying if a variable is a valid ID
#![allow(clippy::unwrap_used)]

use std::sync::LazyLock;

use regex::Regex;

/// Identifier string
pub(crate) static ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_regex() {
        assert!(ID_REGEX.is_match("abc"));
        assert!(ID_REGEX.is_match("myVariable"));
        assert!(!ID_REGEX.is_match("2ndVariable"));
        assert!(!ID_REGEX.is_match("_My Variable"));
        assert!(!ID_REGEX.is_match("variable$"));
        assert!(ID_REGEX.is_match("Another_Variable_123"));
        assert!(ID_REGEX.is_match("a1_b2_c3_d4_e5_f6_g7_h8_i9_j10_k11_l12_m13_n14_o15_p16_q17_r18_s19_t20_u21_v22_w23_x24_y25_z26"));
    }
}
