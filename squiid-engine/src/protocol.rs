// contains JSON structs of transmission protocol objects

use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

/// Server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum MessageAction {
    SendStack,
    SendCommands,
    Quit,
}

/// Client message datatype
/// this is what we recieve from the client
#[derive(Deserialize, Serialize, Debug)]
pub struct ClientMessage {
    pub request_type: RequestType,
    // TODO: be able to send other data structs
    pub payload: String,
}

impl ClientMessage {
    pub fn new(request_type: RequestType, message_payload: String) -> Self {
        Self {
            request_type,
            payload: message_payload,
        }
    }
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

/// Types of messages to be received from the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RequestType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "configuration")]
    Configuration,
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
