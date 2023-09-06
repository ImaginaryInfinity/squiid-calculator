use std::{net::TcpListener, ops::Range};

use nng::{Message, Socket};
use squiid_engine::protocol::{
    client_request::{ClientRequestMessage, RequestPayload, RequestType},
    server_response::ServerResponseMessage,
};
use squiid_parser::{lexer::lex, tokens::Token};

/// Send data to backend
pub fn send_data(socket: &Socket, command: &str) -> ServerResponseMessage {
    let serialized_data: String = serde_json::to_string(&ClientRequestMessage::new(
        RequestType::Input,
        RequestPayload::Input(command.into()),
    ))
    .unwrap();

    let _ = socket.send(serialized_data.as_bytes());
    let msg = socket.recv().unwrap();

    deserialize_message(msg)
}

/// Deserialize a message from the server
fn deserialize_message(msg: Message) -> ServerResponseMessage {
    let msg_string = String::from_utf8(msg.to_vec()).unwrap();
    let data: ServerResponseMessage = serde_json::from_str(&msg_string).unwrap();

    data
}

/// Get current character index based on cursor position and text length
pub fn current_char_index(left_cursor_offset: usize, input_len: usize) -> usize {
    if left_cursor_offset > input_len {
        0
    } else {
        input_len - left_cursor_offset
    }
}

/// Find the first available port in a provided range
pub fn get_available_port(mut range: Range<u16>) -> Option<u16> {
    range.find(|port| port_is_available(*port))
}

/// Test if a specific TCP port is avaiable
fn port_is_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
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
            return *last_token == Token::Float("_") || *last_token == Token::Int("_");
        }
        Err(_) => false,
    }
}
