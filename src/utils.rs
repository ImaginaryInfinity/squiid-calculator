use std::{net::TcpListener, ops::Range};

use nng::{Message, Socket};
use squiid_engine::protocol::{ClientMessage, ServerMessage};

// Send data to backend
pub fn send_data(socket: &Socket, command: &str) -> ServerMessage {
    let serialized_data: String =
        serde_json::to_string(&ClientMessage::new(command.to_owned())).unwrap();

    let _ = socket.send(serialized_data.as_bytes());
    let msg = socket.recv().unwrap();

    deserialize_message(msg)
}

fn deserialize_message(msg: Message) -> ServerMessage {
    let msg_string = String::from_utf8(msg.to_vec()).unwrap();
    let data: ServerMessage = serde_json::from_str(&msg_string).unwrap();

    data
}

// get current character index based on cursor position and text length
pub fn current_char_index(left_cursor_offset: usize, input_len: usize) -> usize {
    let index: usize;
    if left_cursor_offset > input_len {
        index = 0;
    } else {
        index = input_len - left_cursor_offset;
    }

    index
}

// find the first available port in a provided range
pub fn get_available_port(mut range: Range<u16>) -> Option<u16> {
    range.find(|port| port_is_available(*port))
}

// test if a specific TCP port is avaiable
fn port_is_available(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}
