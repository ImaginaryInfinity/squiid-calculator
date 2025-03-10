#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::missing_panics_doc)]

pub mod error;
pub mod lexer;
pub mod parser;
pub mod tokens;

#[cfg(feature = "ffi")]
mod ffi;

use crate::lexer::lex;
use error::ParserError;
use parser::{parse_implicit_multiplication, parse_subtract_sign, shunting_yard_parser};

/// Parse an algebraic string into a vec of tokens in RPN format.
///
/// # Arguments
///
/// * `input` - The string to parse
///
/// # Errors
///
/// If any errors occur while parsing, a [`ParserError`] will be returned
///
/// # Examples
///
/// ```
/// use squiid_parser::parse;
/// use squiid_parser::error::ParserError;
///
/// fn main() -> Result<(), ParserError> {
///     let expected = vec!["3", "6", "4", "6", "*", "+", "*", "5", "/"];
///     let input = "3(6+4*6)/5";
///     assert_eq!(expected, parse(input)?);
///
///     Ok(())
/// }
/// ```
pub fn parse(input: &str) -> Result<Vec<&str>, ParserError> {
    // check for unmatched parenthesis
    if input.matches('(').count() != input.matches(')').count() {
        return Err(ParserError::MismatchedParenthesis);
    }

    let mut tokens = lex(input)?;
    parse_subtract_sign(&mut tokens);
    parse_implicit_multiplication(&mut tokens);
    shunting_yard_parser(tokens)
}
