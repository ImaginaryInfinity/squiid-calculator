use logos::Logos;

use crate::tokens::Token;

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
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let lex = Token::lexer(input).spanned();
    let mut tokens = Vec::new();

    for (token, range) in lex {
        if token.is_err() {
            return Err(format!(
                "Unexpected token: {:?}",
                &input[range.start..range.end]
            ));
        }

        tokens.push(token.map_err(|()| String::from("error unwrapping token"))?);
    }

    Ok(tokens)
}
