use logos::Logos;

use crate::tokens::Token;

/// Lex a given input string
pub fn lex(input: &str) -> Result<Vec<Token>, String> {
    let mut lex = Token::lexer(input).spanned();
    let mut tokens = Vec::new();

    while let Some((token, range)) = lex.next() {
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
