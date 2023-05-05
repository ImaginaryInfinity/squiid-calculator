// contains JSON structs of transmission protocol objects

use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

/// Server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum MessageAction {
    SendStack,
    SendCommands,
}

/// Client message datatype
#[derive(Deserialize, Serialize, Debug)]
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

/// Response struct
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerMessage {
    pub message_type: MessageType,
    pub payload: MessagePayload,
}

impl ServerMessage {
    pub fn new(message_type: MessageType, message_payload: MessagePayload) -> Self {
        Self {
            message_type,
            payload: message_payload,
        }
    }
}

/// Types of messages to send back to the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MessageType {
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
pub enum MessagePayload {
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

/// Macro for getting data out of message payload
#[macro_export]
macro_rules! extract_data {
    ($payload:expr, $variant:path) => {
        match $payload {
            $variant(data) => data,
            _ => panic!("Invalid data type provided for payload: {:?}", $payload),
        }
    };
}
