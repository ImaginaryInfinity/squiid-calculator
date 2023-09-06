use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Client message datatype
/// this is what we recieve from the client
#[derive(Serialize, Debug)]
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

/// configuration deserialization struct
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConfigurationPayload {
    pub action_type: ConfigurationActionType,
    pub section: Option<String>,
    pub key: Option<String>,
}

/// configuration request action types
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

/// custom deserializer for the ClientRequestMessage
/// deserializes "payload" differently depending on the request_type
impl<'de> Deserialize<'de> for ClientRequestMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value: Value = Deserialize::deserialize(deserializer)?;

        match json_value.get("request_type") {
            Some(request_type_value) => {
                let request_type: RequestType = serde_json::from_value(request_type_value.clone())
                    .map_err(serde::de::Error::custom)?;

                match request_type {
                    RequestType::Input => {
                        let payload_str = json_value
                            .get("payload")
                            .and_then(|v| v.as_str())
                            .ok_or_else(|| {
                                serde::de::Error::custom(
                                    "Missing or invalid payload for RequestType::Input",
                                )
                            })?;
                        Ok(ClientRequestMessage {
                            request_type: RequestType::Input,
                            payload: RequestPayload::Input(payload_str.to_string()),
                        })
                    }
                    RequestType::Configuration => {
                        let payload_value = json_value.get("payload").ok_or_else(|| {
                            serde::de::Error::custom(
                                "Missing payload for RequestType::Configuration",
                            )
                        })?;
                        let payload: ConfigurationPayload =
                            serde_json::from_value(payload_value.clone())
                                .map_err(serde::de::Error::custom)?;
                        Ok(ClientRequestMessage {
                            request_type: RequestType::Configuration,
                            payload: RequestPayload::Configuration(payload),
                        })
                    }
                }
            }
            None => Err(serde::de::Error::custom("Missing request_type field")),
        }
    }
}
