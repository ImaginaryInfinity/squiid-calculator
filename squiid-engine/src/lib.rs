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
            "quit" => break,
            recieved => {
                engine.add_item_to_stack(StackableString(recieved.to_string()));
                Ok(())
            },
        };

        match result {
            Ok(()) => {}
            Err(error) => engine
                .add_item_to_stack(StackableString(format!("Error: {:?}", error))),
        }

        // format the stack as a string
        let mut formatted_stack = String::new();
        if engine.stack.len() > 0 {
            for item in &engine.stack {
                match item {
                    StackableFloat(i) => formatted_stack.push_str(&format!("{},", i.to_string())),
                    StackableString(i) => formatted_stack.push_str(&format!("\"{}\",", i)),
                }
            }
            // Remove trailing comma
            if formatted_stack.chars().last().unwrap() == ',' {
                formatted_stack.remove(formatted_stack.len() - 1);
            }
        }

        // respond to client with the stack as a string
        responder.send(&formatted_stack, 0).unwrap();
    }

    // send quit message to client
    responder.send("quit", 0).unwrap();
}
