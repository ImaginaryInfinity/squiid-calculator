pub mod bucket;
pub mod command_mappings;
pub mod config_handler;
pub mod crash_reporter;
pub mod engine;
pub mod utils;

pub mod protocol {
    pub mod client_request;
    pub mod server_response;
}

#[cfg(feature = "ipc")]
pub mod ffi;

use std::{borrow::BorrowMut, panic};

use bucket::Bucket;
use command_mappings::CommandsMap;
use engine::Engine;

#[cfg(feature = "ipc")]
use nng::{Protocol, Socket};
use protocol::{
    client_request::{ConfigurationActionType, ConfigurationPayload},
    server_response::MessageAction,
};

#[cfg(feature = "ipc")]
use crate::{
    protocol::{
        client_request::{RequestPayload, RequestType},
        server_response::{ResponsePayload, ResponseType},
    },
    utils::{recv_data, send_response},
};

#[cfg(feature = "ipc")]
/// The default address to start the server on
const DEFAULT_ADDRESS: &str = "tcp://*:33242";

#[cfg(feature = "ipc")]
/// Start the server at the given address (default is DEFAULT_ADDRESS)
pub fn start_server(address: Option<&str>) {
    //TODO: document features
    #[cfg(not(feature = "disable-crash-reports"))]
    panic::set_hook(Box::new(|panic_info| {
        crash_reporter::crash_report(panic_info, true);

        // propegate panic for frontend to handle
        // TODO: document this
        std::process::exit(2);
    }));

    // Use default address unless one was specified from the command line

    let address_to_bind = match address {
        Some(adr) => adr,
        None => DEFAULT_ADDRESS,
    };

    // create NNG socket to listen on
    let responder = Socket::new(Protocol::Rep0).unwrap();

    // Print and bind to selected port
    assert!(
        responder.listen(address_to_bind).is_ok(),
        "could not bind to address {:?}. if you're using this as a shared object file, are you encoding the input to utf-8 bytes?",
        address_to_bind
    );

    // create message variable and engine
    let mut engine = Engine::new();

    // create hashmap of available commands
    let commands = command_mappings::create_function_map();

    // listen forever
    loop {
        // recieve message from client
        let data = recv_data(&responder);
        // check if error was encountered when parsing JSON
        let recieved = match data {
            Ok(ref value) => value,
            Err(_) => {
                // send error back to client and continue loop
                let _ = send_response(
                    &responder,
                    ResponseType::Error,
                    ResponsePayload::Error("invalid JSON data was sent to the server".to_string()),
                );
                continue;
            }
        };

        let result = match recieved.request_type {
            RequestType::Input => handle_data(
                &mut engine,
                &commands,
                extract_data!(&recieved.payload, RequestPayload::Input),
            ),
            RequestType::Configuration => handle_config_data(
                &mut engine,
                extract_data!(recieved.payload.clone(), RequestPayload::Configuration),
            ),
        };

        match result {
            Ok(MessageAction::SendStack) => {
                let _ = send_response(
                    &responder,
                    ResponseType::Stack,
                    ResponsePayload::Stack(engine.stack.clone()),
                );
            }
            Ok(MessageAction::SendConfigValue(config_value)) => {
                let _ = send_response(
                    &responder,
                    ResponseType::Configuration,
                    ResponsePayload::Configuration(config_value.into()),
                );
            }
            Ok(MessageAction::SendCommands) => {
                let mut avaiable_commands: Vec<String> =
                    commands.keys().map(|k| k.to_owned()).collect();

                // add quit since it is a special case not in the commands list
                avaiable_commands.push(String::from("quit"));

                let _ = send_response(
                    &responder,
                    ResponseType::Commands,
                    ResponsePayload::Commands(avaiable_commands),
                );
            }
            Ok(MessageAction::SendPrevAnswer) => {
                let _ = send_response(
                    &responder,
                    ResponseType::PrevAnswer,
                    ResponsePayload::PrevAnswer(engine.previous_answer.clone()),
                );
            }
            Ok(MessageAction::Quit) => break,
            Err(error) => {
                let _ = send_response(
                    &responder,
                    ResponseType::Error,
                    ResponsePayload::Error(error.to_string()),
                );
            }
        }
    }

    // send quit message to client
    let _ = send_response(
        &responder,
        ResponseType::QuitSig,
        ResponsePayload::QuitSig(None),
    );
}

pub fn handle_data(
    engine: &mut Engine,
    commands: &CommandsMap,
    data: &str,
) -> Result<MessageAction, String> {
    if engine.history.len() > 20 {
        _ = engine.history.pop_front();
        _ = engine.variable_history.pop_front();
    }

    // Don't add to history if command is refresh or commands as it does not affect the stack
    if !["refresh", "commands"].contains(&data) {
        // Add current stack to history
        engine.history.push_back(engine.stack.clone());
        // Add current variable state to history
        engine.variable_history.push_back(engine.variables.clone());
    }

    let result = match commands.get(data) {
        Some(func) => func(engine.borrow_mut()),
        None => {
            // return result value of adding item to stack
            engine.add_item_to_stack(Bucket::from(data.to_string()))
        }
    };

    result
}

/// handle config data sent to the server
pub fn handle_config_data(
    engine: &mut Engine,
    data: ConfigurationPayload,
) -> Result<MessageAction, String> {
    let value_option = match data.action_type {
        ConfigurationActionType::GetKey => {
            if data.section.is_none() {
                return Err("config section not provided in GetKey".to_string());
            }
            if data.key.is_none() {
                return Err("config key not provided in GetKey".to_string());
            }
            engine
                .config
                .get_key(&data.section.unwrap(), &data.key.unwrap())
        }
        ConfigurationActionType::ListSections => engine.config.list_sections(),
        ConfigurationActionType::ListKeys => {
            if data.section.is_none() {
                return Err("config section not provided in ListKeys".to_string());
            }
            engine.config.list_keys(&data.section.unwrap())
        }
        ConfigurationActionType::ListValues => {
            if data.section.is_none() {
                return Err("config section not provided in ListValues".to_string());
            }
            engine.config.list_values(&data.section.unwrap())
        }
        ConfigurationActionType::ListItems => {
            if data.section.is_none() {
                return Err("config section not provided in ListItems".to_string());
            }
            engine.config.list_items(&data.section.unwrap())
        }
        ConfigurationActionType::SetKey => {
            if data.section.is_none() {
                return Err("config section not provided in SetKey".to_string());
            }
            if data.key.is_none() {
                return Err("config key not provided in SetKey".to_string());
            }
            if data.value.is_none() {
                return Err("config value not provided in SetKey".to_string());
            }
            engine.config.set_key(
                &data.section.unwrap(),
                &data.key.unwrap(),
                data.value.unwrap(),
            )
        }
        ConfigurationActionType::CreateSection => {
            if data.section.is_none() {
                return Err("config section not provided in CreateSection".to_string());
            }
            engine.config.create_section(&data.section.unwrap())
        }
        ConfigurationActionType::DeleteSection => {
            if data.section.is_none() {
                return Err("config section not provided in DeleteSection".to_string());
            }
            engine.config.delete_section(&data.section.unwrap())
        }
        ConfigurationActionType::DeleteKey => {
            if data.section.is_none() {
                return Err("config section not provided in DeleteKey".to_string());
            }
            if data.key.is_none() {
                return Err("config key not provided in DeleteKey".to_string());
            }
            engine
                .config
                .delete_key(&data.section.unwrap(), &data.key.unwrap())
        }
    };

    match value_option {
        Ok(item) => Ok(MessageAction::SendConfigValue(item)),
        Err(e) => Err(e),
    }
}
