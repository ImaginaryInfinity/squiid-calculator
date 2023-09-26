pub mod nng;

use crate::protocol::{
    client_request::ClientRequestMessage, server_response::ServerResponseMessage,
};

// TODO: developer docs for this
pub trait IPCBackend {
    /// construct new default object
    fn new() -> Self;

    /// Bind and listen to the given address. Default is defined in lib.rs as DEFAULT_ADDRESS
    fn bind_and_listen(&self, address: &str) -> Result<(), anyhow::Error>;

    /// Recieve data from the client
    fn recv_data(&self) -> Result<ClientRequestMessage, anyhow::Error>;

    // Send data to the client
    fn send_data(&self, response: ServerResponseMessage) -> Result<(), anyhow::Error>;
}
