use std::{borrow::BorrowMut, collections::HashMap};

use nng::Socket;

use crate::{
    engine::Engine,
    protocol::{ClientMessage, MessageAction, MessagePayload, MessageType, ServerMessage},
};

// function to check if a string is numeric (includes floats)
pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // If a character is not a number or contains only a decimal point, negative sign, or e, the string is not numeric
        if !c.is_numeric()
            && !(['.', '-', 'e'].contains(&c)
                && str.chars().count() > 1
                && !['-', 'e'].contains(&(str.chars().last().unwrap())))
        {
            return false;
        }
    }
    return true;
}

pub fn send_response(
    socket: &Socket,
    response_type: MessageType,
    response_payload: MessagePayload,
) -> Result<(), serde_json::Error> {
    let server_response = ServerMessage::new(response_type, response_payload);

    let json = serde_json::to_string(&server_response)?;

    socket.send(json.as_bytes()).unwrap();
    Ok(())
}

pub fn recv_data(socket: &Socket) -> Result<ClientMessage, serde_json::Error> {
    // recieve data from client
    let msg = socket.recv().unwrap();
    // Convert received message to a string
    let recieved = std::str::from_utf8(&msg).unwrap();

    let client_response: ClientMessage = serde_json::from_str(recieved)?;
    Ok(client_response)
}

// function map stuff for creating a hashmap of available functions
macro_rules! function_map_entry {
    ($function_map:expr,$name:expr,$func_name:ident) => {
        $function_map.insert(
            String::from($name),
            Box::new(|engine: &mut Engine| engine.borrow_mut().$func_name()) as Box<EngineFunction>,
        )
    };
}

type EngineFunction = dyn Fn(&mut Engine) -> Result<MessageAction, String>;

pub fn create_function_map() -> HashMap<String, Box<EngineFunction>> {
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
        Box::new(|_engine: &mut Engine| Ok(MessageAction::SendStack)),
    );

    function_map
}
