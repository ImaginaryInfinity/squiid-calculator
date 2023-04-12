use squiid_parser::{
    parse,
    utils::{is_string_alphabetic, is_string_numeric},
};

#[test]
fn test_classifier_functions() {
    // test numeric function
    assert_eq!(is_string_numeric("abc"), false);
    assert_eq!(is_string_numeric("1a"), false);
    assert_eq!(is_string_numeric("12e.a"), false);
    assert_eq!(is_string_numeric("123"), true);
    assert_eq!(is_string_numeric("1.2"), true);
    assert_eq!(is_string_numeric("1.2e7"), true);

    // test alphabetic function
    assert_eq!(is_string_alphabetic("abc"), true);
    assert_eq!(is_string_alphabetic("1a"), false);
    assert_eq!(is_string_alphabetic("12e.a"), false);
    assert_eq!(is_string_alphabetic("123"), false);
    assert_eq!(is_string_alphabetic("1.2"), false);
    assert_eq!(is_string_alphabetic("1.2e7"), false);
}

#[test]
fn test_parsing() {
    assert_eq!(
        parse(
            "sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^_6)*25500)^_5.2656)+1)^0.286-1))"
        ),
        vec![
            "5", "1", "0.2", "350", "661.5", "/", "2", "^", "*", "+", "3.5", "^", "1", "-", "1",
            "6.875", "10", "_6", "^", "*", "25500", "*", "-", "_5.2656", "^", "*", "1", "+",
            "0.286", "^", "1", "-", "*", "sqrt"
        ]
    );

    assert_eq!(parse("$A * $B + $C"), vec!["$A", "$B", "*", "$C", "+"]);

    assert_eq!(parse("$A + $B * $C"), vec!["$A", "$B", "$C", "*", "+"]);

    assert_eq!(parse("$A * ($B + $C)"), vec!["$A", "$B", "$C", "+", "*"]);

    assert_eq!(
        parse("34 * 5.3 ^ 2 + 0.9"),
        vec!["34", "5.3", "2", "^", "*", "0.9", "+"]
    );

    assert_eq!(
        parse("8e3 * ($B + 4.532 * _0.2) + $A"),
        vec!["8e3", "$B", "4.532", "_0.2", "*", "+", "*", "$A", "+"]
    );
}
