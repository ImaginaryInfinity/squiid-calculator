// contains JSON structs of transmission protocol objects

use serde::Serialize;

use crate::bucket::Bucket;

// server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum ResponseAction {
    SendStack,
    SendCommands,
}

// response struct
#[derive(Serialize)]
pub struct ServerResponse {
    response_type: ResponseType,
    payload: ResponsePayload,
}

impl ServerResponse {
    pub fn new(response_type: ResponseType, response_payload: ResponsePayload) -> Self {
        Self {
            response_type: response_type,
            payload: response_payload,
        }
    }
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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
