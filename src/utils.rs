use squiid_parser::{lexer::lex, tokens::Token};

/// Get current character index based on cursor position and text length
pub fn current_char_index(left_cursor_offset: usize, input_len: usize) -> usize {
    input_len.saturating_sub(left_cursor_offset)
}

/// Test if a str buffer is scientific notation
pub fn input_buffer_is_sci_notate(buffer: &str) -> bool {
    // TODO: if RPN input can ever have a negative number, handle that

    // preliminary checks
    if buffer.is_empty() {
        return false;
    }

    if !buffer.ends_with('e') {
        return false;
    }

    match lex(buffer.trim_end_matches('e')) {
        Ok(tokens) => {
            // test if the last token before the trailing 'e' is an int or float
            let last_token = &tokens[tokens.len() - 1];
            *last_token == Token::Float("_") || *last_token == Token::Int("_")
        }
        Err(_) => false,
    }
}
