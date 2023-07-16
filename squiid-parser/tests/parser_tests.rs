use squiid_parser::{
    parse,
    parser::{parse_implicit_multiplication, parse_subtract_sign},
    tokens::Token::{self, *},
};

fn parse_and_compare(input: &str, expected_tokens: Vec<&str>) {
    let tokens = parse(input).unwrap();
    assert_eq!(tokens, expected_tokens);
}

#[test]
fn test_parse_subtract_sign() {
    // Test case 1: Subtract sign at the beginning of an expression
    let mut tokens = vec![Subtract("-"), Int("3"), Add("+"), Int("6")];
    parse_subtract_sign(&mut tokens);
    assert_eq!(tokens, vec![Negative("-"), Int("3"), Add("+"), Int("6")]);

    // Test case 2: Subtract sign at the beginning of an opening parenthesis
    let mut tokens = vec![LParen("("), Subtract("-"), Int("3"), Add("+"), Int("6")];
    parse_subtract_sign(&mut tokens);
    assert_eq!(
        tokens,
        vec![LParen("("), Negative("-"), Int("3"), Add("+"), Int("6")]
    );

    // Test case 3: Subtract sign after another operator
    let mut tokens = vec![Int("3"), Add("+"), Subtract("-"), Int("5")];
    parse_subtract_sign(&mut tokens);
    assert_eq!(tokens, vec![Int("3"), Add("+"), Negative("-"), Int("5")]);

    // Test case 4: Subtract sign as an argument in a function
    let mut tokens = vec![
        Function("function("),
        Int("3"),
        Comma(","),
        Subtract("-"),
        Int("3"),
        RParen(")"),
        Add("+"),
        Int("6"),
    ];
    parse_subtract_sign(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Function("function("),
            Int("3"),
            Comma(","),
            Negative("-"),
            Int("3"),
            RParen(")"),
            Add("+"),
            Int("6")
        ]
    );
}

#[test]
fn test_parse_implicit_multiplication() {
    // Test case 1: Implicit multiplication between a constant and a variable
    let mut tokens = vec![Token::Constant("5"), Token::VariableRecal("$x")];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::Constant("5"),
            Token::Multiply("*"),
            Token::VariableRecal("$x"),
        ]
    );

    // Test case 2: Implicit multiplication between a variable and a function
    let mut tokens = vec![
        Token::VariableRecal("$x"),
        Token::Function("sin("),
        Token::Int("30"),
        Token::RParen(")"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::VariableRecal("$x"),
            Token::Multiply("*"),
            Token::Function("sin("),
            Token::Int("30"),
            Token::RParen(")"),
        ]
    );

    // Test case 3: No implicit multiplication needed
    let mut tokens = vec![
        Token::Constant("3"),
        Token::Add("+"),
        Token::VariableRecal("$y"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::Constant("3"),
            Token::Add("+"),
            Token::VariableRecal("$y"),
        ]
    );

    // Test case 4: Implicit multiplication between an int and a function
    let mut tokens = vec![
        Token::Int("2"),
        Token::Function("log("),
        Token::Int("10"),
        Token::RParen(")"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::Int("2"),
            Token::Multiply("*"),
            Token::Function("log("),
            Token::Int("10"),
            Token::RParen(")"),
        ]
    );

    // Test case 5: Implicit multiplication between a constant and a function
    let mut tokens = vec![
        Token::Constant("#pi"),
        Token::Function("log("),
        Token::Int("10"),
        Token::RParen(")"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::Constant("#pi"),
            Token::Multiply("*"),
            Token::Function("log("),
            Token::Int("10"),
            Token::RParen(")"),
        ]
    );

    // Test case 6: Implicit multiplication between two sets of parenthesis
    let mut tokens = vec![
        Token::LParen("("),
        Token::Int("10"),
        Token::RParen(")"),
        Token::LParen("("),
        Token::Int("6"),
        Token::RParen(")"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::LParen("("),
            Token::Int("10"),
            Token::RParen(")"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::Int("6"),
            Token::RParen(")"),
        ]
    );

    // Test case 7: Implicit multiplication between one set of parenthesis
    let mut tokens = vec![
        Token::Int("10"),
        Token::LParen("("),
        Token::Int("6"),
        Token::RParen(")"),
    ];
    parse_implicit_multiplication(&mut tokens);
    assert_eq!(
        tokens,
        vec![
            Token::Int("10"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::Int("6"),
            Token::RParen(")"),
        ]
    );
}

#[test]
fn test_shunting_yard_parser() {
    parse_and_compare("5 + 3 * 2", vec!["5", "3", "2", "*", "+"]);
    parse_and_compare("(5 + 3) * 2", vec!["5", "3", "+", "2", "*"]);
    parse_and_compare("sin(3 * 2) + 4", vec!["3", "2", "*", "sin", "4", "+"]);
    parse_and_compare(
        "5 * 3 + 2 * 4 / 6",
        vec!["5", "3", "*", "2", "4", "*", "6", "/", "+"],
    );
    parse_and_compare("5 + 3 * sin(2)", vec!["5", "3", "2", "sin", "*", "+"]);
    parse_and_compare(
        "sqrt(4 + 9) * log(100)",
        vec!["4", "9", "+", "sqrt", "100", "log", "*"],
    );
    parse_and_compare(
        "3 ^ 2 - 4 * sqrt(16)",
        vec!["3", "2", "^", "4", "16", "sqrt", "*", "-"],
    );
    parse_and_compare(
        "sin(cos(tan(1))) + blog(2, 8)",
        vec!["1", "tan", "cos", "sin", "2", "8", "blog", "+"],
    );
    parse_and_compare(
        "(2 + 3) * 4 - abs(-5)",
        vec!["2", "3", "+", "4", "*", "5", "chs", "abs", "-"],
    );
    parse_and_compare(
        "sqrt(5 + 3 * sin(2)) / blog(2, 8)",
        vec![
            "5", "3", "2", "sin", "*", "+", "sqrt", "2", "8", "blog", "/",
        ],
    );
    parse_and_compare(
        "2 ^ (3 + abs(-4)) * max(5, min(6, 7))",
        vec![
            "2", "3", "4", "chs", "abs", "+", "^", "5", "6", "7", "min", "max", "*",
        ],
    );
    parse_and_compare(
        "sin(cos(2 * #PI / 3)) + 1 / (log(10) - 3.5)",
        vec![
            "2", "#PI", "*", "3", "/", "cos", "sin", "1", "10", "log", "3.5", "-", "/", "+",
        ],
    );
    parse_and_compare(
        "3 * (4 + 5) / (6 - abs(-7) * (8 + 9))",
        vec![
            "3", "4", "5", "+", "*", "6", "7", "chs", "abs", "8", "9", "+", "*", "-", "/",
        ],
    );
    parse_and_compare(
        "abs(-1 + 2) * ceil(sqrt(16)) - round(3.14159)",
        vec![
            "1", "chs", "2", "+", "abs", "16", "sqrt", "ceil", "*", "3.14159", "round", "-",
        ],
    );
    parse_and_compare(
        "sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^-6)*25500)^-5.2656)+1)^0.286-1))",
        vec![
            "5", "1", "0.2", "350", "661.5", "/", "2", "^", "*", "+", "3.5", "^", "1", "-", "1",
            "6.875", "10", "6", "chs", "^", "*", "25500", "*", "-", "5.2656", "chs", "^", "*", "1",
            "+", "0.286", "^", "1", "-", "*", "sqrt",
        ],
    );

    parse_and_compare("$A * $B + $C", vec!["$A", "$B", "*", "$C", "+"]);

    parse_and_compare("$A + $B * $C", vec!["$A", "$B", "$C", "*", "+"]);

    parse_and_compare("$A * ($B + $C)", vec!["$A", "$B", "$C", "+", "*"]);

    parse_and_compare(
        "34 * 5.3 ^ 2 + 0.9",
        vec!["34", "5.3", "2", "^", "*", "0.9", "+"],
    );

    parse_and_compare(
        "8e3 * ($B + 4.532 * -0.2) + $A",
        vec!["8e3", "$B", "4.532", "0.2", "chs", "*", "+", "*", "$A", "+"],
    );
}
