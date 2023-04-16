use crate::lex::lex;
use tokens::Token;

mod lex;
mod tokens;

pub fn parse(input: &str) -> Vec<Token> {
    lex(input).unwrap()
}
