mod ffi;
pub mod lexer;
pub mod parser;
pub mod tokens;

use crate::lexer::lex;
use parser::{parse_implicit_multiplication, parse_subtract_sign, shunting_yard_parser};

/// Parse an input string into a Vec
pub fn parse(input: &str) -> Result<Vec<&str>, String> {
    // check for unmatched parenthesis
    if input.matches('(').count() != input.matches(')').count() {
        return Err("Mismatched parentheses: Unmatched closing parenthesis".to_string());
    }

    let mut tokens = lex(input)?;
    parse_subtract_sign(&mut tokens);
    parse_implicit_multiplication(&mut tokens);
    shunting_yard_parser(tokens)
}
