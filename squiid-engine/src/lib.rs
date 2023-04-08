mod bucket;
mod engine;
mod utils;

use std::{collections::HashMap, borrow::BorrowMut};

use engine::Engine;

use crate::bucket::Bucket;

macro_rules! function_map_entry {
    ($function_map:expr,$name:expr,$func_name:ident) => {
        $function_map.insert(
            String::from($name),
            Box::new(|engine: &mut Engine| engine.borrow_mut().$func_name()) as Box<EngineFunction>
        )
    };
}

type EngineFunction = dyn Fn(&mut Engine) -> Result<(), &'static str>;

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

    // manually insert refresh since it doesn't use an engine method
    function_map.insert(String::from("refresh"), Box::new(|_engine: &mut Engine| Ok(())));

    function_map
}

pub fn start_server(address: Option<&str>) {
    // create zeromq socket to listen on
    let context = zmq::Context::new();
    let responder = context.socket(zmq::REP).unwrap();

    // Use default address unless one was specified from the command line
    let address_to_bind = match address {
        Some(adr) => adr,
        None => "tcp://*:33242",
    };

    // Print and bind to selected port
    // println!("{} {}", "Bound to: ", address_to_bind);
    assert!(responder.bind(address_to_bind).is_ok());

    // create message variable and engine
    let mut msg = zmq::Message::new();
    let mut engine = Engine::new();

    // listen forever
    loop {

        if engine.history.len() > 20 {
            _ = engine.history.pop_front();
        }

        // recieve message from client
        responder.recv(&mut msg, 0).unwrap();
        // Convert received message to a string
        let recieved = msg.as_str().unwrap();

        // Don't add to history if command is refresh as it does not affect the stack
        if recieved != "refresh" {
            // Add current stack to history
            engine.history.push_back(engine.stack.clone());
        }

        // create hashmap of available commands
        let commands = create_function_map();

        let result = match recieved {
            "quit" => break,
            recieved => 
                match commands.get(recieved) {
                    Some(func) => func(engine.borrow_mut()),
                    None => {
                        let _ = engine.add_item_to_stack(Bucket::from(recieved.to_string()));
                        Ok(())
                    }
                }
        };

        let mut formatted_response = String::new();
        match result {
            Ok(()) => {
                // format the stack as a string
                if engine.stack.len() > 0 {
                    for item in &engine.stack {
                        // TODO: make this better
                        formatted_response.push_str(&format!("{},", item.to_string()))
                    }
                    // Remove trailing comma
                    if formatted_response.chars().last().unwrap() == ',' {
                        formatted_response.remove(formatted_response.len() - 1);
                    }
                }
            }
            Err(error) => {
                formatted_response = format!("Error: {}", error.to_string());
            }
        }

        // respond to client with the stack as a string
        responder.send(&formatted_response, 0).unwrap();
    }

    // send quit message to client
    responder.send("quit", 0).unwrap();
}
