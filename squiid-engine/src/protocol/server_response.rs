use serde::{Deserialize, Serialize};

use crate::bucket::Bucket;

/// Server response type for internal handling
#[derive(Debug, PartialEq)]
pub enum MessageAction {
    SendStack,
    SendCommands,
    SendConfigValue(ConfigValue),
    Quit,
}

#[derive(Debug, PartialEq)]
pub enum ConfigValue {
    Value(toml::Value),
    StringList(Vec<String>),
    ValueList(Vec<toml::Value>),
    KeyValueList(Vec<(String, toml::Value)>),
    None(()),
}

impl Into<serde_json::Value> for ConfigValue {
    fn into(self) -> serde_json::Value {
        match self {
            ConfigValue::Value(item) => toml_to_serde_json_value(&item),
            ConfigValue::StringList(string_list) => serde_json::Value::Array(
                string_list
                    .into_iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
            ConfigValue::ValueList(toml_value_list) => serde_json::Value::Array(
                toml_value_list
                    .into_iter()
                    .map(|v| toml_to_serde_json_value(&v))
                    .collect(),
            ),
            ConfigValue::KeyValueList(key_value_list) => serde_json::Value::Object(
                key_value_list
                    .into_iter()
                    .map(|(k, v)| (k.clone(), toml_to_serde_json_value(&v)))
                    .collect(),
            ),
            ConfigValue::None(_) => serde_json::Value::Null,
        }
    }
}

// Convert a toml Value to a serde_json Value
fn toml_to_serde_json_value(toml_value: &toml::Value) -> serde_json::Value {
    match toml_value {
        toml::Value::String(s) => serde_json::Value::String(s.clone()),
        toml::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        toml::Value::Float(f) => serde_json::Value::from(*f),
        toml::Value::Boolean(b) => serde_json::Value::Bool(*b),
        toml::Value::Datetime(dt) => serde_json::Value::String(dt.to_string()),
        toml::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|v| toml_to_serde_json_value(v)).collect())
        }
        toml::Value::Table(table) => {
            let object: serde_json::Map<String, serde_json::Value> = table
                .iter()
                .map(|(k, v)| (k.clone(), toml_to_serde_json_value(v)))
                .collect();
            serde_json::Value::Object(object)
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
    #[serde(rename = "configuration")]
    Configuration,
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
    #[serde(rename = "configuration")]
    Configuration(serde_json::Value),
}
