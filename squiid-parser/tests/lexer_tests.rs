use squiid_parser::lexer::lex;
use squiid_parser::tokens::Token;

fn tokenize_and_compare(input: &str, expected_tokens: Vec<Token>) {
    let tokens = lex(input).unwrap();
    assert_eq!(tokens, expected_tokens);
}

#[test]
fn test_complex_tokenization() {
    // Test complex expressions
    tokenize_and_compare("foo()", vec![Token::Function("foo("), Token::RParen(")")]);

    tokenize_and_compare(
        "bar(3, 4.5)",
        vec![
            Token::Function("bar("),
            Token::Int("3"),
            Token::Comma(","),
            Token::Float("4.5"),
            Token::RParen(")"),
        ],
    );

    tokenize_and_compare(
        "$x = 2 * $y + 3.14e-2",
        vec![
            Token::VariableRecal("$x"),
            Token::Equal("="),
            Token::Int("2"),
            Token::Multiply("*"),
            Token::VariableRecal("$y"),
            Token::Add("+"),
            Token::ScientificNotation("3.14e-2"),
        ],
    );

    tokenize_and_compare(
        "3.14 + 2.5 * 4 - 1 / 5",
        vec![
            Token::Float("3.14"),
            Token::Add("+"),
            Token::Float("2.5"),
            Token::Multiply("*"),
            Token::Int("4"),
            Token::Subtract("-"),
            Token::Int("1"),
            Token::Divide("/"),
            Token::Int("5"),
        ],
    );

    tokenize_and_compare(
        "(3.14 + 2.5) * (4 - 1) / 5",
        vec![
            Token::LParen("("),
            Token::Float("3.14"),
            Token::Add("+"),
            Token::Float("2.5"),
            Token::RParen(")"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::Int("4"),
            Token::Subtract("-"),
            Token::Int("1"),
            Token::RParen(")"),
            Token::Divide("/"),
            Token::Int("5"),
        ],
    );

    tokenize_and_compare(
        "2 * $x + 3 ^ 4",
        vec![
            Token::Int("2"),
            Token::Multiply("*"),
            Token::VariableRecal("$x"),
            Token::Add("+"),
            Token::Int("3"),
            Token::Power("^"),
            Token::Int("4"),
        ],
    );

    tokenize_and_compare(
        "sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^-6)*25500)^-5.2656)+1)^0.286-1))",
        vec![
            Token::Function("sqrt("),
            Token::Int("5"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::LParen("("),
            Token::LParen("("),
            Token::LParen("("),
            Token::LParen("("),
            Token::Int("1"),
            Token::Add("+"),
            Token::Float("0.2"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::Int("350"),
            Token::Divide("/"),
            Token::Float("661.5"),
            Token::RParen(")"),
            Token::Power("^"),
            Token::Int("2"),
            Token::RParen(")"),
            Token::Power("^"),
            Token::Float("3.5"),
            Token::Subtract("-"),
            Token::Int("1"),
            Token::RParen(")"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::Int("1"),
            Token::Subtract("-"),
            Token::LParen("("),
            Token::Float("6.875"),
            Token::Multiply("*"),
            Token::Int("10"),
            Token::Power("^"),
            Token::Subtract("-"),
            Token::Int("6"),
            Token::RParen(")"),
            Token::Multiply("*"),
            Token::Int("25500"),
            Token::RParen(")"),
            Token::Power("^"),
            Token::Subtract("-"),
            Token::Float("5.2656"),
            Token::RParen(")"),
            Token::Add("+"),
            Token::Int("1"),
            Token::RParen(")"),
            Token::Power("^"),
            Token::Float("0.286"),
            Token::Subtract("-"),
            Token::Int("1"),
            Token::RParen(")"),
            Token::RParen(")"),
        ],
    );

    tokenize_and_compare(
        "$A * $B + $C",
        vec![
            Token::VariableRecal("$A"),
            Token::Multiply("*"),
            Token::VariableRecal("$B"),
            Token::Add("+"),
            Token::VariableRecal("$C"),
        ],
    );

    tokenize_and_compare(
        "$A + $B * $C",
        vec![
            Token::VariableRecal("$A"),
            Token::Add("+"),
            Token::VariableRecal("$B"),
            Token::Multiply("*"),
            Token::VariableRecal("$C"),
        ],
    );

    tokenize_and_compare(
        "$A * ($B + $C)",
        vec![
            Token::VariableRecal("$A"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::VariableRecal("$B"),
            Token::Add("+"),
            Token::VariableRecal("$C"),
            Token::RParen(")"),
        ],
    );

    tokenize_and_compare(
        "34 * 5.3 ^ 2 + 0.9",
        vec![
            Token::Int("34"),
            Token::Multiply("*"),
            Token::Float("5.3"),
            Token::Power("^"),
            Token::Int("2"),
            Token::Add("+"),
            Token::Float("0.9"),
        ],
    );

    tokenize_and_compare(
        "8e3 * ($B + 4.532 * -0.2) + $A",
        vec![
            Token::ScientificNotation("8e3"),
            Token::Multiply("*"),
            Token::LParen("("),
            Token::VariableRecal("$B"),
            Token::Add("+"),
            Token::Float("4.532"),
            Token::Multiply("*"),
            Token::Subtract("-"),
            Token::Float("0.2"),
            Token::RParen(")"),
            Token::Add("+"),
            Token::VariableRecal("$A"),
        ],
    );
}

#[test]
fn test_individual_tokenization() {
    // Test individual tokens
    tokenize_and_compare(",", vec![Token::Comma(",")]);
    tokenize_and_compare("@", vec![Token::PrevAns("@")]);
    tokenize_and_compare("(", vec![Token::LParen("(")]);
    tokenize_and_compare(")", vec![Token::RParen(")")]);
    tokenize_and_compare("^", vec![Token::Power("^")]);
    tokenize_and_compare("*", vec![Token::Multiply("*")]);
    tokenize_and_compare("/", vec![Token::Divide("/")]);
    tokenize_and_compare("%", vec![Token::Modulo("%")]);
    tokenize_and_compare("+", vec![Token::Add("+")]);
    tokenize_and_compare("-", vec![Token::Subtract("-")]);
}

#[test]
fn test_scientific_notation() {
    // Test scientific notation
    tokenize_and_compare("3.14e0", vec![Token::ScientificNotation("3.14e0")]);

    tokenize_and_compare("2.5e+3", vec![Token::ScientificNotation("2.5e+3")]);

    tokenize_and_compare("1.23e-4", vec![Token::ScientificNotation("1.23e-4")]);

    tokenize_and_compare("6.02e23", vec![Token::ScientificNotation("6.02e23")]);

    tokenize_and_compare("9.8E-6", vec![Token::ScientificNotation("9.8E-6")]);

    tokenize_and_compare("1.0e0", vec![Token::ScientificNotation("1.0e0")]);
}
