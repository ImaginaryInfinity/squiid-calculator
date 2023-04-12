pub mod bucket;
pub mod engine;
pub mod protocol;
pub mod utils;
pub mod exposed;

use std::borrow::BorrowMut;

use bucket::Bucket;
use engine::Engine;
use protocol::ResponseAction;

use nng::{Message, Protocol, Socket};

use crate::{
    protocol::{ResponsePayload, ResponseType},
    utils::send_response,
};

const DEFAULT_ADDRESS: &'static str = "tcp://*:33242";

pub fn start_server(address: Option<&str>) {
    // Use default address unless one was specified from the command line
    let address_to_bind = match address {
        Some(adr) => adr,
        None => DEFAULT_ADDRESS,
    };

    // create zeromq socket to listen on
    let responder = Socket::new(Protocol::Rep0).unwrap();

    // Print and bind to selected port
    assert!(
        responder.listen(address_to_bind).is_ok(),
        "could not bind to address {:?}. if you're using this as a shared object file, are you encoding the input to utf-8 bytes?",
        address_to_bind
    );

    // create message variable and engine
    let mut msg: Message;
    let mut engine = Engine::new();

    // listen forever
    loop {
        if engine.history.len() > 20 {
            _ = engine.history.pop_front();
            _ = engine.variable_history.pop_front();
        }

        // recieve message from client
        msg = responder.recv().unwrap();
        // Convert received message to a string
        let recieved = std::str::from_utf8(&msg).unwrap();

        // Don't add to history if command is refresh or commands as it does not affect the stack
        if !["refresh", "commands"].contains(&recieved) {
            // Add current stack to history
            engine.history.push_back(engine.stack.clone());
            // Add current variable state to history
            engine.variable_history.push_back(engine.variables.clone());
        }

        // create hashmap of available commands
        let commands = utils::create_function_map();

        let result = match recieved {
            "quit" => break,
            recieved => match commands.get(recieved) {
                Some(func) => func(engine.borrow_mut()),
                None => {
                    // return result value of adding item to stack
                    engine.add_item_to_stack(Bucket::from(recieved.to_string()))
                }
            },
        };

        match result {
            Ok(ResponseAction::SendStack) => {
                let _ = send_response(
                    &responder,
                    ResponseType::Stack,
                    ResponsePayload::Stack(engine.stack.clone()),
                );
            }
            Ok(ResponseAction::SendCommands) => {
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
