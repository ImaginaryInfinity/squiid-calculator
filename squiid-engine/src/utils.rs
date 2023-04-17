use lazy_static::lazy_static;
use nng::Socket;
use regex::Regex;

use crate::protocol::{ClientMessage, MessagePayload, MessageType, ServerMessage};

lazy_static! {
    pub static ref ID_REGEX: regex::Regex = Regex::new(r"[_a-zA-Z][_0-9a-zA-Z]*").unwrap();
}

// function to check if a string is numeric (includes floats)
pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // If a character is not a number or contains only a decimal point, negative sign, or e, the string is not numeric
        if !c.is_numeric()
            && !(['.', '-', 'e'].contains(&c)
                && str.chars().count() > 1
                && !['-', 'e'].contains(&(str.chars().last().unwrap())))
        {
            return false;
        }
    }
    return true;
}

pub fn send_response(
    socket: &Socket,
    response_type: MessageType,
    response_payload: MessagePayload,
) -> Result<(), serde_json::Error> {
    let server_response = ServerMessage::new(response_type, response_payload);

    let json = serde_json::to_string(&server_response)?;

    socket.send(json.as_bytes()).unwrap();
    Ok(())
}

pub fn recv_data(socket: &Socket) -> Result<ClientMessage, serde_json::Error> {
    // recieve data from client
    let msg = socket.recv().unwrap();
    // Convert received message to a string
    let recieved = std::str::from_utf8(&msg).unwrap();

    let client_response: ClientMessage = serde_json::from_str(recieved)?;
    Ok(client_response)
}
