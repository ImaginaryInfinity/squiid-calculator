// contains JSON structs of transmission protocol objects

use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

// server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum MessageAction {
    SendStack,
    SendCommands,
}

// client message datatype
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

// response struct
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MessagePayload {
    #[serde(rename = "stack")]
    Stack(Vec<Bucket>),
    #[serde(rename = "commands")]
    Commands(Vec<String>),
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "quitsig")]
    QuitSig(Option<u8>), // this should always be set to none
}

// macro for getting data out of message payload
#[macro_export]
macro_rules! extract_data {
    ($payload:expr, $variant:path) => {
        match $payload {
            $variant(data) => data,
            _ => panic!("Invalid data type provided for payload: {:?}", $payload),
        }
    };
}
