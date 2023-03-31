mod engine;
mod stackable_items;
mod utils;

use engine::Engine;
use stackable_items::StackableItems::{StackableFloat, StackableString};

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
        // Add current stack to history
        engine.history.push_back(engine.stack.clone());

        if engine.history.len() > 20{
            _ = engine.history.pop_front();
        }

        // recieve message from client
        responder.recv(&mut msg, 0).unwrap();
        // Convert received message to a string
        let recieved = msg.as_str().unwrap();

        let result = match recieved {
            "add" => engine.add(),
            "subtract" => engine.subtract(),
            "multiply" => engine.multiply(),
            "divide" => engine.divide(),
            "power" => engine.power(),
            "sqrt" => engine.sqrt(),
            "mod" => engine.modulo(),
            "sin" => engine.sin(),
            "cos" => engine.cos(),
            "tan" => engine.tan(),
            "sec" => engine.sec(),
            "csc" => engine.csc(),
            "cot" => engine.cot(),
            "asin" => engine.asin(),
            "acos" => engine.acos(),
            "atan" => engine.atan(),
            "log" => engine.log(),
            "logb" => engine.logb(),
            "ln" => engine.ln(),
            "abs" => engine.abs(),
            "eq" => engine.eq(),
            "gt" => engine.gt(),
            "lt" => engine.lt(),
            "gte" => engine.gte(),
            "lte" => engine.lte(),
            "round" => engine.round(),
            "invert" => engine.invert(),
            "drop" => engine.drop(),
            "swap" => engine.swap(),
            "dup" => engine.dup(),
            "rolldown" => engine.roll_down(),
            "rollup" => engine.roll_up(),
            "store" => engine.store(),
            "clear" => engine.clear(),
            "refresh" => {Ok(())},
            "undo" => engine.undo(),
            "quit" => break,
            recieved => {
                let _ = engine.add_item_to_stack(StackableString(recieved.to_string()));
                Ok(())
            },
        };

        let mut formatted_response = String::new();
        match result {
            Ok(()) => {
                // format the stack as a string
                if engine.stack.len() > 0 {
                    for item in &engine.stack {
                        match item {
                            StackableFloat(i) => formatted_response.push_str(&format!("{},", i.to_string())),
                            StackableString(i) => formatted_response.push_str(&format!("\"{}\",", i)),
                        }
                    }
                    // Remove trailing comma
                    if formatted_response.chars().last().unwrap() == ',' {
                        formatted_response.remove(formatted_response.len() - 1);
                    }
                }
            }
            Err(error) => {
                formatted_response=format!("Error: {}", error.to_string());
            },
        }


        // respond to client with the stack as a string
        responder.send(&formatted_response, 0).unwrap();
    }

    // send quit message to client
    responder.send("quit", 0).unwrap();
}
