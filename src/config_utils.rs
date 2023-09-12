use serde_json::Value;
use squiid_engine::{
    extract_data,
    protocol::{
        client_request::{
            ConfigurationActionType, ConfigurationPayload, RequestPayload, RequestType,
        },
        server_response::ResponsePayload,
    },
};

use crate::{
    app::{update_stack_or_error, App},
    utils::send_data,
};

/// a collection of utilities for constructing configuration JSON requests

/// send config data to the server and return the response
#[allow(dead_code)]
fn send_configuration_data(app: &mut App, config_payload: ConfigurationPayload) -> Value {
    // create payload
    let payload = RequestPayload::Configuration(config_payload);

    // send data
    let response = send_data(app.socket, RequestType::Configuration, payload);
    // error handling
    update_stack_or_error(response.clone(), app);

    let payload_data = extract_data!(response.payload, ResponsePayload::Configuration);
    payload_data
}

/// Get a key from the config
#[allow(dead_code)]
pub fn get_key(app: &mut App, section: &str, key: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::GetKey,
            Some(section.into()),
            Some(key.into()),
            None,
        ),
    )
}

/// List sections from the config
#[allow(dead_code)]
pub fn list_sections(app: &mut App) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(ConfigurationActionType::ListSections, None, None, None),
    )
}

/// List keys from a specific section
#[allow(dead_code)]
pub fn list_keys(app: &mut App, section: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::ListKeys,
            Some(section.into()),
            None,
            None,
        ),
    )
}

/// List values from the config
#[allow(dead_code)]
pub fn list_values(app: &mut App, section: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::ListValues,
            Some(section.into()),
            None,
            None,
        ),
    )
}

/// List items from the config
#[allow(dead_code)]
pub fn list_items(app: &mut App, section: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::ListItems,
            Some(section.into()),
            None,
            None,
        ),
    )
}

/// Set a specific key in the config
#[allow(dead_code)]
pub fn set_key(app: &mut App, section: &str, key: &str, value: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::ListValues,
            Some(section.into()),
            Some(key.into()),
            Some(value.into()),
        ),
    )
}

/// Create a section in the config
#[allow(dead_code)]
pub fn create_section(app: &mut App, section: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::CreateSection,
            Some(section.into()),
            None,
            None,
        ),
    )
}

/// Delete a specific section in the config
#[allow(dead_code)]
pub fn delete_section(app: &mut App, section: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::DeleteSection,
            Some(section.into()),
            None,
            None,
        ),
    )
}

/// Delete a specific key in the config
#[allow(dead_code)]
pub fn delete_key(app: &mut App, section: &str, key: &str) -> Value {
    send_configuration_data(
        app,
        ConfigurationPayload::new(
            ConfigurationActionType::DeleteKey,
            Some(section.into()),
            Some(key.into()),
            None,
        ),
    )
}
