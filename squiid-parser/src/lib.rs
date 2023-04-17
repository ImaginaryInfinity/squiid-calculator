use crate::lex::lex;
use parser::shunting_yard_parser;

mod lex;
mod parser;
mod tokens;
mod exposed;

pub fn parse(input: &str) -> Result<Vec<&str>, String> {
    // check for unmatched parenthesis
    if input.matches('(').count() != input.matches(')').count() {
        return Err("Mismatched parentheses: Unmatched closing parenthesis".to_string());
    }

    let tokens = lex(input)?;
    shunting_yard_parser(tokens)
}
