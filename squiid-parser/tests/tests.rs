use squiid_parser::parse;

#[test]
fn test_parsing() {
    assert_eq!(
        parse(
            "sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^_6)*25500)^_5.2656)+1)^0.286-1))"
        ),
        vec!["5", "1", "0.2", "350", "661.5", "divide", "2", "power", "multiply", "add", "3.5", "power", "1", "subtract", "1", "6.875", "10", "6", "chs", "power", "multiply", "25500", "multiply", "subtract", "5.2656", "chs", "power", "multiply", "1", "add", "0.286", "power", "1", "subtract", "multiply", "sqrt"]
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
