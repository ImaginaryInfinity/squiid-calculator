pub mod bucket;
pub mod command_mappings;
pub mod engine;
pub mod ffi;
pub mod protocol;
pub mod utils;

use std::borrow::BorrowMut;

use bucket::Bucket;
use engine::Engine;
use protocol::MessageAction;

use nng::{Protocol, Socket};

use crate::{
    protocol::{MessagePayload, MessageType},
    utils::{recv_data, send_response},
};

/// The default address to start the server on
const DEFAULT_ADDRESS: &'static str = "tcp://*:33242";

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
        if engine.history.len() > 20 {
            _ = engine.history.pop_front();
            _ = engine.variable_history.pop_front();
        }

        // recieve message from client
        let data = recv_data(&responder);
        // check if error was encountered when parsing JSON
        let recieved = match data {
            Ok(ref value) => &*value.payload,
            Err(_) => {
                // send error back to client and continue loop
                let _ = send_response(
                    &responder,
                    MessageType::Error,
                    MessagePayload::Error("invalid JSON data was sent to the server".to_string()),
                );
                continue;
            }
        };

        // Don't add to history if command is refresh or commands as it does not affect the stack
        if !["refresh", "commands"].contains(&recieved) {
            // Add current stack to history
            engine.history.push_back(engine.stack.clone());
            // Add current variable state to history
            engine.variable_history.push_back(engine.variables.clone());
        }

        // TODO: protocol implementation of setting the previous answer variable
        // unless theres a better way, this should add support for just typing in
        // a variable or number and having that be the previous answer
        let result = match recieved {
            "quit" => break,
            recieved => match commands.get(recieved) {
                Some(func) => func(engine.borrow_mut()),
                None => {
                    // return result value of adding item to stack
                    engine.add_item_to_stack(Bucket::from(recieved.to_string()), false)
                }
            },
        };

        match result {
            Ok(MessageAction::SendStack) => {
                let _ = send_response(
                    &responder,
                    MessageType::Stack,
                    MessagePayload::Stack(engine.stack.clone()),
                );
            }
            Ok(MessageAction::SendCommands) => {
                let mut avaiable_commands: Vec<String> =
                    commands.keys().map(|k| k.to_owned()).collect();

                // add quit since it is a special case not in the commands list
                avaiable_commands.push(String::from("quit"));

                let _ = send_response(
                    &responder,
                    MessageType::Commands,
                    MessagePayload::Commands(avaiable_commands),
                );
            }
            Err(error) => {
                let _ = send_response(
                    &responder,
                    MessageType::Error,
                    MessagePayload::Error(error.to_string()),
                );
            }
        }
    }

    // send quit message to client
    let _ = send_response(
        &responder,
        MessageType::QuitSig,
        MessagePayload::QuitSig(None),
    );
}
