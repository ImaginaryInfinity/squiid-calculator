//! Squiid Parser is the algebraic expression parser for [Squiid Calculator](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid), designed
//! to parse infix notation into postifix (RPN) notation, which can then be evaluated by [squiid-engine](https://crates.io/crates/squiid-engine).
//!
//! This module provides functionality for lexing, parsing, and handling algebraic expressions,
//! supporting implicit multiplication and correct operator precedence using the Shunting Yard algorithm.
//!
//! ## Modules
//!
//! - [`error`]: Defines error types encountered during parsing.
//! - [`lexer`]: Handles lexical analysis, converting input strings into tokens.
//! - [`parser`]: Implements parsing logic, including handling implicit multiplication and operator precedence.
//! - [`tokens`]: Defines token structures used throughout the parsing process.
//!
//! ## Functionality
//!
//! The primary function exposed is [`parse`], which converts an infix algebraic string
//! into a vector of tokens in RPN format. It ensures proper handling of operators,
//! parentheses, and implicit multiplication.
//!
//! ## Features
//!
//! - **FFI Support** (optional): Enables C-compatible parsing via the `ffi` module.
//! - **Strict Safety Guidelines**: Uses `#![deny(clippy::unwrap_used)]` and related lints
//!   to enforce error handling best practices.
//!
//! ## Usage
//!
//! ```
//! use squiid_parser::parse;
//! use squiid_parser::error::ParserError;
//!
//! fn main() -> Result<(), ParserError> {
//!     let expected = vec!["3", "6", "4", "6", "*", "+", "*", "5", "/"];
//!     let input = "3(6+4*6)/5";
//!     assert_eq!(expected, parse(input)?);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! If parsing fails, an appropriate [`ParserError`] is returned, such as
//! `MismatchedParenthesis` when parentheses are unbalanced.

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
