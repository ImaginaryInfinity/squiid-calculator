pub mod bucket;
pub mod command_mappings;
pub mod engine;
pub mod utils;
pub mod config_handler;

pub mod protocol {
    pub mod client_request;
    pub mod server_response;
}

#[cfg(feature = "ipc")]
pub mod ffi;

use std::borrow::BorrowMut;

use bucket::Bucket;
use command_mappings::CommandsMap;
use engine::Engine;

#[cfg(feature = "ipc")]
use nng::{Protocol, Socket};
use protocol::server_response::MessageAction;

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
            RequestType::Configuration => engine.handle_config_data(extract_data!(recieved.payload, RequestPayload::Configuration)),
        };

        // set previous answer
        let _ = engine.update_previous_answer();

        match result {
            Ok(MessageAction::SendStack) => {
                let _ = send_response(
                    &responder,
                    ResponseType::Stack,
                    ResponsePayload::Stack(engine.stack.clone()),
                );
            }
            Ok(MessageAction::SendConfigValue) => {
                let _ = send_response(&responder, ResponseType::Configuration, ResponsePayload::Configuration(()))
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
