use thiserror::Error;

#[derive(Error, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ParserError {
    #[error("Unexpected token: {0}")]
    UnexpectedToken(String),
    #[error("Trailing negative sign")]
    TrailingNegative,
    #[error("Mismatched parentheses: Unmatched closing parenthesis")]
    MismatchedParenthesis,
}
