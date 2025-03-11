use logos::Logos;

use crate::{error::ParserError, tokens::Token};

/// Lex a given input string into tokens
///
/// You will most likely never have to use this unless you're doing some weird preprocessing
/// validation stuff.
///
/// # Arguments
///
/// * `input` - The input string to tokenize
///
/// # Errors
///
/// An error may arise if an unexpected token is encountered
pub fn lex(input: &str) -> Result<Vec<Token>, ParserError> {
    let lex = Token::lexer(input).spanned();
    let mut tokens = Vec::new();

    for (token, range) in lex {
        if let Ok(value) = token {
            tokens.push(value);
        } else {
            return Err(ParserError::UnexpectedToken(
                input[range.start..range.end].to_string(),
            ));
        }
    }

    Ok(tokens)
}
