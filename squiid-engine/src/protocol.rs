// contains JSON structs of transmission protocol objects

use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

// server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum ResponseAction {
    SendStack,
    SendCommands,
}

// response struct
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerResponse {
    pub response_type: ResponseType,
    pub payload: ResponsePayload,
}

impl ServerResponse {
    pub fn new(response_type: ResponseType, response_payload: ResponsePayload) -> Self {
        Self {
            response_type: response_type,
            payload: response_payload,
        }
    }
}

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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ResponsePayload {
    #[serde(rename = "stack")]
    Stack(Vec<Bucket>),
    #[serde(rename = "commands")]
    Commands(Vec<String>),
    #[serde(rename = "error")]
    Error(String),
    #[serde(rename = "quitsig")]
    QuitSig(Option<u8>), // this should always be set to none
}

// macro for getting data out of response payload
#[macro_export]
macro_rules! extract_data {
    ($payload:expr, $variant:path) => {
        match $payload {
            $variant(data) => data,
            _ => panic!("Invalid data type provided for payload: {:?}", $payload),
        }
    };
}
