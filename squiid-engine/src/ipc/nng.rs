use nng::{Protocol, Socket};

use crate::protocol::client_request::ClientRequestMessage;

use super::IPCBackend;

pub struct NanoMsg {
    socket: nng::Socket,
}

impl IPCBackend for NanoMsg {
    fn new() -> Self {
        Self {
            socket: Socket::new(Protocol::Rep0).unwrap(),
        }
    }

    fn bind_and_listen(&self, address: &str) -> Result<(), anyhow::Error> {
        self.socket.listen(address)?;
        Ok(())
    }

    fn recv_data(
        &self,
    ) -> Result<crate::protocol::client_request::ClientRequestMessage, anyhow::Error> {
        // recieve data from client
        let msg = self.socket.recv()?;

        // Convert received message to a string
        let recieved = std::str::from_utf8(&msg)?;
        let client_response: ClientRequestMessage = serde_json::from_str(recieved)?;
        Ok(client_response)
    }

    fn send_data(
        &self,
        response: crate::protocol::server_response::ServerResponseMessage,
    ) -> Result<(), anyhow::Error> {
        let json = serde_json::to_string(&response)?;

        match self.socket.send(json.as_bytes()) {
            Ok(it) => it,
            Err(err) => return Err(err.1.into()),
        };
        Ok(())
    }
}
