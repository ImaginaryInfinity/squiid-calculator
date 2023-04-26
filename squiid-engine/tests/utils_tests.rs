use squiid_engine::utils::{ID_REGEX, NUMERIC_REGEX};

#[test]
fn test_numeric_regex() {
    assert_eq!(NUMERIC_REGEX.is_match("abc"), false);
    assert_eq!(NUMERIC_REGEX.is_match("1a"), false);
    assert_eq!(NUMERIC_REGEX.is_match("12e.a"), false);
    assert_eq!(NUMERIC_REGEX.is_match("123"), true);
    assert_eq!(NUMERIC_REGEX.is_match("1.2"), true);
    assert_eq!(NUMERIC_REGEX.is_match("1.2e7"), true);
}

#[test]
fn test_id_regex() {
    assert_eq!(ID_REGEX.is_match("abc"), true);
    assert_eq!(ID_REGEX.is_match("myVariable"), true);
    assert_eq!(ID_REGEX.is_match("2ndVariable"), false);
    assert_eq!(ID_REGEX.is_match("_My Variable"), false);
    assert_eq!(ID_REGEX.is_match("variable$"), false);
    assert_eq!(ID_REGEX.is_match("Another_Variable_123"), true);
    assert_eq!(ID_REGEX.is_match("a1_b2_c3_d4_e5_f6_g7_h8_i9_j10_k11_l12_m13_n14_o15_p16_q17_r18_s19_t20_u21_v22_w23_x24_y25_z26"), true);
}
