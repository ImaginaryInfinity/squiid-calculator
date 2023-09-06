use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

/// Server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum MessageAction {
    SendStack,
    SendCommands,
    Quit,
}

/// Response struct
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerResponseMessage {
    pub response_type: ResponseType,
    pub payload: ResponsePayload,
}

impl ServerResponseMessage {
    pub fn new(response_type: ResponseType, message_payload: ResponsePayload) -> Self {
        Self {
            response_type,
            payload: message_payload,
        }
    }
}

/// Types of messages to send back to the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponseType {
    #[serde(rename = "stack")]
    Stack,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "commands")]
    Commands,
    #[serde(rename = "quitsig")]
    QuitSig,
}

/// Types of message payloads to send to the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponsePayload {
    #[serde(rename = "stack")]
    Stack(Vec<Bucket>),
    #[serde(rename = "commands")]
    Commands(Vec<String>),
    #[serde(rename = "error")]
    Error(String),
    /// This should always be set to None
    #[serde(rename = "quitsig")]
    QuitSig(Option<u8>),
}
