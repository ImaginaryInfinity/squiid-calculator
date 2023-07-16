use lazy_static::lazy_static;
use regex::Regex;

#[cfg(feature = "ipc")]
use crate::protocol::{ClientMessage, MessagePayload, MessageType, ServerMessage};
#[cfg(feature = "ipc")]
use nng::Socket;

lazy_static! {
    /// Identifier string
    pub static ref ID_REGEX: Regex = Regex::new(r"^[_a-zA-Z][_0-9a-zA-Z]*$").unwrap();
    /// Numeric string
    pub static ref NUMERIC_REGEX: Regex =
        Regex::new(r"^[-]?(?:[0-9]*\.?[0-9]+(?:[eE][-+]?\d+(?:\.\d+)?)?|[0-9]+)$").unwrap();
}

#[cfg(feature = "ipc")]
/// Send a response to the client
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

#[cfg(feature = "ipc")]
/// Recieve data from the client
pub fn recv_data(socket: &Socket) -> Result<ClientMessage, serde_json::Error> {
    // recieve data from client
    let msg = socket.recv().unwrap();
    // Convert received message to a string
    let recieved = std::str::from_utf8(&msg).unwrap();

    let client_response: ClientMessage = serde_json::from_str(recieved)?;
    Ok(client_response)
}
