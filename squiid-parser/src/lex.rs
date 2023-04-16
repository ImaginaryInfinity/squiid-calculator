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

const RIGHT_SIDE_IMPLICIT: [Token; 9] = [
    Function("_"),
    VariableRecal("_"),
    Constant("_"),
    ScientificNotation("_"),
    Float("_"),
    Int("_"),
    PrevAns("_"),
    LParen("_"),
    Negative("_"),
];

// lex a given input string
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut lex = Token::lexer(input).spanned().peekable();
    let mut tokens = Vec::new();
    let mut index = 0;

    // for (token, range) in lex {
    while let Some((token, range)) = lex.next() {
        if token.is_err() {
            return Err(format!(
                "Unexpected token: {:?}",
                &input[range.start..range.end]
            ));
        }

        let cur_token = token.unwrap();

        match cur_token {
            Token::Subtract(_) => {
                // parse whether this is a negative sign or a minus operator
                // it is a negative sign if:
                //
                // at the beginning of an expression
                // at the beginning of an opening parenthesis (-3+6)
                // after another operator (3+-5, 3*-5, 3^-5)
                // as an argument in a function, so after a comma (function(3, -3))

                // at the beginning of an expression
                if index == 0 {
                    tokens.push(Token::Negative("-"));
                } else {
                    match *tokens.last().unwrap() {
                        // at the beginning of an opening parenthesis (-3+6)
                        Token::RParen("(") |
                        // after another operator (3+-5, 3*-5, 3^-5)
                        Token::Add("+") | Token::Subtract("-") | Token::Modulo("%") | Token::Multiply("*") | Token::Divide("/") | Token::Power("^") | Token::Equal("=") |
                        // as an argument in a function, so after a comma (function(3, -3))
                        Token::Comma(",") => {
                            tokens.push(Token::Negative("-"));
                        },
                        _ => tokens.push(Token::Subtract("-")),
                    }
                }
            }
            // Left side (current token):
            // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, RParen
            //
            // Right Side (peek token):
            // Function, VariableRecal, Constant, ScientificNotation, Float, Int, PrevAns, LParen, Negative
            //
            // Implicit multiplication happens if something on the left side list is followed by something on the right side list
            token if LEFT_SIDE_IMPLICIT.contains(&token) => {
                // push current token to tokens list
                tokens.push(token);

                match lex.peek() {
                    Some(token) => match &token.0 {
                        Ok(value) => {
                            // there is a next token that is not an error, test if in right side multiplication table
                            if RIGHT_SIDE_IMPLICIT.contains(value) {
                                // implicit multiplication is needed
                                tokens.push(Multiply("*"));
                            }
                        }
                        Err(_) => {
                            return Err(format!(
                                "Unexpected token: {:?}",
                                &input[token.1.start..token.1.end]
                            ))
                        }
                    },
                    None => {
                        // no next token, do nothing
                    }
                };
            }
            item => tokens.push(item),
        }

        // manual tracking of index since enumerate() moves lex
        index += 1;
    }

    Ok(tokens)
}
