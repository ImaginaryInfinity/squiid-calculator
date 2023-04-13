// contains JSON structs of transmission protocol objects

use serde::Serialize;

// response struct
#[derive(Serialize, Debug)]
pub struct ClientMessage {
    pub payload: String,
}

impl ClientMessage {
    pub fn new(message_payload: String) -> Self {
        Self {
            payload: message_payload,
        }
    }
}
