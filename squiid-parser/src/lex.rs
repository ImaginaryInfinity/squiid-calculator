use logos::Logos;

use crate::tokens::Token::{self, *};

const LEFT_SIDE_IMPLICIT: [Token; 7] = [
    VariableRecal("_"),
    Constant("_"),
    ScientificNotation("_"),
    Float("_"),
    Int("_"),
    PrevAns("_"),
    RParen("_"),
];

const RIGHT_SIDE_IMPLICIT: [Token; 8] = [
    Function("_"),
    VariableRecal("_"),
    Constant("_"),
    ScientificNotation("_"),
    Float("_"),
    Int("_"),
    PrevAns("_"),
    LParen("_"),
];

fn parse_subtract_sign(tokens: &mut Vec<Token>) {
    // parse whether this is a negative sign or a minus operator
    // it is a negative sign if:
    //
    // at the beginning of an expression
    // at the beginning of an opening parenthesis (-3+6)
    // after another operator (3+-5, 3*-5, 3^-5)
    // as an argument in a function, so after a comma (function(3, -3))

    let mut negative_replacements: Vec<usize> = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if *token == Subtract("-") {
            // at the beginning of an expression
            if index == 0 {
                negative_replacements.push(index);
            } else {
                // get the token before the current negative sign
                match tokens[index - 1] {
                    // at the beginning of an opening parenthesis (-3+6)
                    Token::LParen("(") |
                    // after another operator (3+-5, 3*-5, 3^-5)
                    Token::Add("+") | Token::Subtract("-") | Token::Modulo("%") | Token::Multiply("*") | Token::Divide("/") | Token::Power("^") | Token::Equal("=") |
                    // as an argument in a function, so after a comma (function(3, -3))
                    Token::Comma(",") => {
                        negative_replacements.push(index);
                    },
                    _ => (),
                }
            }
        }
    }

    // do replacements
    for index in negative_replacements {
        tokens[index] = Negative("-");
    }
}

fn parse_implicit_multiplication(tokens: &mut Vec<Token>) {
    // Left side (current token):
    // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, RParen
    //
    // Right Side (peek token):
    // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, LParen
    //
    // Implicit multiplication happens if something on the left side list is followed by something on the right side list

    let mut multiply_insertions = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if LEFT_SIDE_IMPLICIT.contains(&token) {
            // make sure that we aren't on the last token
            if index + 1 != tokens.len() {
                // there is a next token that is not an error, test if in right side multiplication table
                if RIGHT_SIDE_IMPLICIT.contains(&tokens[index + 1]) {
                    // implicit multiplication is needed at the index in front of the current token
                    multiply_insertions.push(index + 1);
                }
            };
        }
    }

    // do insertions
    for (index, insertion_index) in multiply_insertions.iter().enumerate() {
        // offset by index to take account for tokens vec growing in size
        tokens.insert(*insertion_index + index, Multiply("*"));
    }
}

// lex a given input string
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut lex = Token::lexer(input).spanned();
    let mut tokens = Vec::new();

    while let Some((token, range)) = lex.next() {
        if token.is_err() {
            return Err(format!(
                "Unexpected token: {:?}",
                &input[range.start..range.end]
            ));
        }

        tokens.push(token.unwrap());
    }

    parse_subtract_sign(&mut tokens);
    parse_implicit_multiplication(&mut tokens);

    Ok(tokens)
}
