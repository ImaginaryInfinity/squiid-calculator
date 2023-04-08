use squiid_engine::utils::is_string_numeric;

#[test]
fn test_numeric_function() {
    assert_eq!(is_string_numeric("abc"), false);
    assert_eq!(is_string_numeric("1a"), false);
    assert_eq!(is_string_numeric("12e.a"), false);
    assert_eq!(is_string_numeric("123"), true);
    assert_eq!(is_string_numeric("1.2"), true);
    assert_eq!(is_string_numeric("1.2e7"), true);
}
