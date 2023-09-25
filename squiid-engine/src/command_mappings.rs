use std::{borrow::BorrowMut, collections::HashMap};

use crate::{engine::Engine, protocol::server_response::MessageAction};

/// Insert a function and reference name into a hashmap
macro_rules! function_map_entry {
    ($function_map:expr,$name:expr,$func_name:ident) => {
        $function_map.insert(
            String::from($name),
            Box::new(|engine: &mut Engine| engine.borrow_mut().$func_name()) as Box<EngineFunction>,
        )
    };
}

type EngineFunction = dyn Fn(&mut Engine) -> Result<MessageAction, String>;
pub type CommandsMap = HashMap<String, Box<dyn Fn(&mut Engine) -> Result<MessageAction, String>>>;

/// Create a map of every available function and it's respective command
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
    function_map_entry!(function_map, "blog", blog);
    function_map_entry!(function_map, "ln", ln);
    function_map_entry!(function_map, "abs", abs);
    function_map_entry!(function_map, "eq", eq);
    function_map_entry!(function_map, "gt", gt);
    function_map_entry!(function_map, "lt", lt);
    function_map_entry!(function_map, "leq", geq);
    function_map_entry!(function_map, "geq", leq);
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
    function_map_entry!(function_map, "redo", redo);
    function_map_entry!(function_map, "commands", list_commands);
    function_map_entry!(function_map, "quit", quit);
    function_map_entry!(
        function_map,
        "update_previous_answer",
        update_previous_answer
    );

    // manually insert refresh since it doesn't use an engine method
    function_map.insert(
        String::from("refresh"),
        Box::new(|_engine: &mut Engine| Ok(MessageAction::SendStack)),
    );

    function_map
}
