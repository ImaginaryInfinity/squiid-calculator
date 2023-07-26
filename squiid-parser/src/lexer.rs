use logos::Logos;

use crate::tokens::Token;

/// Lex a given input string
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

        tokens.push(token.unwrap());
    }

    Ok(tokens)
}
