pub mod bucket;
pub mod engine;
pub mod protocol;
pub mod utils;

use std::{borrow::BorrowMut, collections::HashMap};

use bucket::Bucket;
use engine::Engine;
use protocol::ResponseAction;

use nng::{Message, Protocol, Socket};

use crate::{
    protocol::{ResponsePayload, ResponseType},
    utils::send_response,
};

const DEFAULT_ADDRESS: &'static str = "tcp://*:33242";

macro_rules! function_map_entry {
    ($function_map:expr,$name:expr,$func_name:ident) => {
        $function_map.insert(
            String::from($name),
            Box::new(|engine: &mut Engine| engine.borrow_mut().$func_name()) as Box<EngineFunction>,
        )
    };
}

type EngineFunction = dyn Fn(&mut Engine) -> Result<ResponseAction, String>;

fn create_function_map() -> HashMap<String, Box<EngineFunction>> {
    let mut function_map = HashMap::new();

    // Insert string keys and function objects into the hashmap

    function_map_entry!(function_map, "add", add);
    function_map_entry!(function_map, "subtract", subtract);
    function_map_entry!(function_map, "divide", divide);
    function_map_entry!(function_map, "multiply", multiply);
    function_map_entry!(function_map, "power", power);
    function_map_entry!(function_map, "sqrt", sqrt);
    function_map_entry!(function_map, "mod", modulo);
    function_map_entry!(function_map, "sin", sin);
    function_map_entry!(function_map, "cos", cos);
    function_map_entry!(function_map, "tan", tan);
    function_map_entry!(function_map, "sec", sec);
    function_map_entry!(function_map, "csc", csc);
    function_map_entry!(function_map, "cot", cot);
    function_map_entry!(function_map, "asin", asin);
    function_map_entry!(function_map, "acos", acos);
    function_map_entry!(function_map, "atan", atan);
    function_map_entry!(function_map, "log", log);
    function_map_entry!(function_map, "logb", logb);
    function_map_entry!(function_map, "ln", ln);
    function_map_entry!(function_map, "abs", abs);
    function_map_entry!(function_map, "eq", eq);
    function_map_entry!(function_map, "gt", gt);
    function_map_entry!(function_map, "lt", lt);
    function_map_entry!(function_map, "gte", gte);
    function_map_entry!(function_map, "lte", lte);
    function_map_entry!(function_map, "round", round);
    function_map_entry!(function_map, "invert", invert);
    function_map_entry!(function_map, "chs", chs);
    function_map_entry!(function_map, "drop", drop);
    function_map_entry!(function_map, "swap", swap);
    function_map_entry!(function_map, "dup", dup);
    function_map_entry!(function_map, "rolldown", roll_down);
    function_map_entry!(function_map, "rollup", roll_up);
    function_map_entry!(function_map, "store", store);
    function_map_entry!(function_map, "purge", purge);
    function_map_entry!(function_map, "invstore", invstore);
    function_map_entry!(function_map, "clear", clear);
    function_map_entry!(function_map, "clear", clear);
    function_map_entry!(function_map, "undo", undo);
    function_map_entry!(function_map, "commands", list_commands);

    // manually insert refresh since it doesn't use an engine method
    function_map.insert(
        String::from("refresh"),
        Box::new(|_engine: &mut Engine| Ok(ResponseAction::SendStack)),
    );

    function_map
}

pub fn start_server(address: Option<&str>) {
    // create zeromq socket to listen on
    let responder = Socket::new(Protocol::Rep0).unwrap();

    // Use default address unless one was specified from the command line
    let address_to_bind = match address {
        Some(adr) => adr,
        None => DEFAULT_ADDRESS,
    };

    // Print and bind to selected port
    assert!(responder.listen(address_to_bind).is_ok());

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
        let commands = create_function_map();

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
