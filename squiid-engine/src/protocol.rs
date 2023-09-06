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
pub struct ClientRequestMessage {
    pub request_type: RequestType,
    #[serde(flatten)]
    pub payload: RequestPayload,
}

impl ClientRequestMessage {
    pub fn new(request_type: RequestType, message_payload: RequestPayload) -> Self {
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

/// cofniguration request action types
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ConfigurationActionType {
    #[serde(rename = "get_key")]
    GetKey,
    #[serde(rename = "list_sections")]
    ListSections,
    #[serde(rename = "list_keys")]
    ListKeys,
    #[serde(rename = "list_values")]
    ListValues,
    #[serde(rename = "list_items")]
    ListItems,
    #[serde(rename = "set_key")]
    SetKey,
    #[serde(rename = "create_section")]
    CreateSection,
    #[serde(rename = "delete_section")]
    DeleteSection,
    #[serde(rename = "delete_key")]
    DeleteKey,
}

/// configuration deserialization struct
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConfigurationPayload {
    pub action_type: ConfigurationActionType,
    pub section: Option<String>,
    pub key: Option<String>,
}

/// Types of messages to be received from the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RequestType {
    #[serde(rename = "input")]
    Input,
    #[serde(rename = "configuration")]
    Configuration,
}

/// Types of message payloads to be received from the client
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RequestPayload {
    #[serde(rename = "payload")]
    Input(String),
    #[serde(rename = "payload")]
    Configuration(ConfigurationPayload),
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
