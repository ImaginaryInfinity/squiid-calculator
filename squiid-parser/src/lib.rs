use crate::lex::lex;
use parser::shunting_yard_parser;

mod lex;
mod tokens;
mod parser;

pub fn parse(input: &str) -> Vec<&str> {
    shunting_yard_parser(lex(input).unwrap())
}
