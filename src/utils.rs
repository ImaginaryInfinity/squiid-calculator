use std::{net::TcpListener, ops::Range};

use zmq::Socket;

// Send data to backend
pub fn send_data(socket: &Socket, command: &str) -> String {
    let mut msg = zmq::Message::new();
    let _ = socket.send(command, 0);
    let _ = socket.recv(&mut msg, 0);
    msg.as_str().unwrap().to_string()
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
