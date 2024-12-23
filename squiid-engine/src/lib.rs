pub mod bucket;
pub mod command_mappings;
pub mod engine;
pub mod utils;

#[cfg(feature = "crash-reporting")]
pub mod crash_reporter;

pub mod protocol {
    pub mod client_request;
    pub mod server_response;
}

#[cfg(feature = "ipc")]
pub mod ffi;

#[cfg(feature = "ipc")]
pub mod ipc;

use std::{borrow::BorrowMut, panic};

use std::path::PathBuf;

use bucket::Bucket;
use command_mappings::CommandsMap;
use engine::Engine;

#[cfg(feature = "ipc")]
use crate::protocol::{
    client_request::RequestPayload,
    server_response::{MessageAction, ResponsePayload, ResponseType},
};

#[cfg(feature = "ipc")]
/// The default address to start the server on
const DEFAULT_ADDRESS: &str = "tcp://*:33242";

#[cfg(feature = "ipc")]
/// Start the server at the given address (default is DEFAULT_ADDRESS)
pub fn start_server(address: Option<&str>, crash_report_directory: Option<PathBuf>) {
    //TODO: document features

    use protocol::server_response::ServerResponseMessage;

    use crate::ipc::IPCBackend;
    #[cfg(feature = "crash-reporting")]
    panic::set_hook(Box::new(move |panic_info| {
        crash_reporter::crash_report(panic_info, crash_report_directory.clone());

        // propegate panic for frontend to handle
        // TODO: document this
        std::process::exit(2);
    }));

    // Use default address unless one was specified from the command line

    let address_to_bind = match address {
        Some(adr) => adr,
        None => DEFAULT_ADDRESS,
    };

    // create nng IPC wrapper
    let ipc_wrapper = ipc::nng::NanoMsg::new();

    // Print and bind to selected port
    assert!(
        ipc_wrapper.bind_and_listen(address_to_bind).is_ok(),
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
        let data = ipc_wrapper.recv_data();
        // check if error was encountered when parsing JSON
        let recieved = match data {
            Ok(ref value) => value,
            Err(_) => {
                // send error back to client and continue loop
                let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
                    ResponseType::Error,
                    ResponsePayload::Error("invalid JSON data was sent to the server".to_string()),
                ));
                continue;
            }
        };

        let result = match &recieved.payload {
            RequestPayload::Input(val) => handle_data(&mut engine, &commands, val),
        };

        match result {
            Ok(MessageAction::SendStack) => {
                let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
                    ResponseType::Stack,
                    ResponsePayload::Stack(engine.stack.clone()),
                ));
            }
            Ok(MessageAction::SendCommands) => {
                let avaiable_commands: Vec<String> =
                    commands.keys().map(|k| k.to_owned()).collect();

                let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
                    ResponseType::Commands,
                    ResponsePayload::Commands(avaiable_commands),
                ));
            }
            Ok(MessageAction::SendPrevAnswer) => {
                let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
                    ResponseType::PrevAnswer,
                    ResponsePayload::PrevAnswer(engine.previous_answer.clone()),
                ));
            }
            Ok(MessageAction::Quit) => break,
            Err(error) => {
                let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
                    ResponseType::Error,
                    ResponsePayload::Error(error.to_string()),
                ));
            }
        }
    }

    // send quit message to client
    let _ = ipc_wrapper.send_data(ServerResponseMessage::new(
        ResponseType::QuitSig,
        ResponsePayload::QuitSig(None),
    ));
}

pub fn handle_data(
    engine: &mut Engine,
    commands: &CommandsMap,
    data: &str,
) -> Result<MessageAction, String> {
    if engine.undo_history.len() > 20 {
        _ = engine.undo_history.pop_front();
        _ = engine.undo_variable_history.pop_front();
    }

    // Don't add to history if command is refresh, commands, or update_previous_answer as it does not affect the stack
    if ![
        "refresh",
        "commands",
        "update_previous_answer",
        "undo",
        "redo",
    ]
    .contains(&data)
    {
        // reset everything in front of the undo history pointer
        engine.undo_history.drain(
            engine
                .undo_history
                .len()
                .saturating_sub(engine.undo_state_pointer as usize)..,
        );
        engine.undo_variable_history.drain(
            engine
                .undo_variable_history
                .len()
                .saturating_sub(engine.undo_state_pointer as usize)..,
        );
        // reset history pointer
        engine.undo_state_pointer = 0;

        // Add current stack to history
        engine.undo_history.push_back(engine.stack.clone());
        // Add current variable state to history
        engine
            .undo_variable_history
            .push_back(engine.variables.clone());
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
