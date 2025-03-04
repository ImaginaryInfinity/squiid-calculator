use squiid_engine::utils::ID_REGEX;

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
