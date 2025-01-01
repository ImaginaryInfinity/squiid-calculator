pub mod bucket;
pub mod command_mappings;
pub mod engine;
pub mod utils;

#[cfg(feature = "crash-reporting")]
pub mod crash_reporter;

#[cfg(feature = "ffi")]
pub mod ffi;

use std::{
    borrow::BorrowMut,
    sync::{LazyLock, Mutex},
};

use bucket::Bucket;
use command_mappings::CommandsMap;
use engine::Engine;

static ENGINE: LazyLock<Mutex<Engine>> = LazyLock::new(|| Mutex::new(Engine::new()));
static COMMAND_MAPPINGS: LazyLock<CommandsMap> =
    LazyLock::new(command_mappings::create_function_map);

/// Server signal type for internal handling
#[derive(Debug, PartialEq)]
pub enum EngineSignal {
    /// The stack was updated
    StackUpdated,
    /// A quit was requested
    Quit,
    /// No operation
    NOP,
}

/// This function is an abstraction which allows you to run one RPN operation on an engine.
///
/// # Arguments
///
/// * `engine` - The engine to use
/// * `data` - The data (command or number) to execute.
///
/// # Errors
///
/// When the command which was input creates an invalid state in the engine, such as when an
/// undefined variable is referenced.
pub fn handle_data(engine: &mut Engine, data: &str) -> Result<EngineSignal, String> {
    if engine.undo_history.len() > 20 {
        _ = engine.undo_history.pop_front();
        _ = engine.undo_variable_history.pop_front();
    }

    // Don't add to history if command is refresh, undo, or redo as it does not affect the stack
    if !["refresh", "undo", "redo"].contains(&data) {
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

    let result = match COMMAND_MAPPINGS.get(data) {
        Some(func) => func(engine.borrow_mut()),
        None => {
            // return result value of adding item to stack
            engine.add_item_to_stack(Bucket::from(data.to_string()))
        }
    };

    result
}

/// Struct to identify which EngineSignals were triggered during the submission of multiple
/// commands to the engine (usually in `execute_rpn_data`)
#[derive(Debug, Default, Clone)]
pub struct EngineSignalSet {
    /// This is set if the `get_stack` method should be called to retrieve the new stack
    stack_updated: bool,
    /// This is set if the frontend should quit
    quit: bool,
    /// This is set if there was an error while putting data into the engine
    error: Option<String>,
}

impl EngineSignalSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Given a result of an action, merge it into the set. This is a convinience method to easily
    /// set fields if the client should get a data structure from the engine.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to merge into the set
    pub fn merge(&mut self, action: Result<EngineSignal, String>) {
        match action {
            Ok(v) => match v {
                EngineSignal::StackUpdated => self.stack_updated = true,
                EngineSignal::Quit => self.quit = true,
                EngineSignal::NOP => (),
            },
            Err(e) => self.error = Some(e),
        }
    }

    /// Whether or not the stack has been updated
    pub fn stack_updated(&self) -> bool {
        self.stack_updated
    }

    /// Whether or not the frontend should quit
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Get the last encountered error if available
    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// Execute multiple RPN commands in the engine at once.
///
/// # Arguments
///
/// * `rpn_data` - The list of RPN data to execute
///
/// # Errors
///
/// This function errors if locking the engine mutex fails
pub fn execute_multiple_rpn(rpn_data: Vec<&str>) -> EngineSignalSet {
    let mut engine = ENGINE.lock().unwrap();

    let mut engine_signals = EngineSignalSet::new();

    for item in rpn_data {
        // submit each piece of data to the engine
        let response = handle_data(&mut engine, item);
        // merge the response into the actions set
        engine_signals.merge(response);

        // if an error was encountered, terminate early
        if engine_signals.get_error().is_some() {
            return engine_signals;
        }
    }

    engine_signals
}

/// Execute a single RPN statement
#[macro_export]
macro_rules! execute_single_rpn {
    ($i:expr) => {
        execute_multiple_rpn(vec![$i])
    };
}

/// Get the current stack from the engine.
///
/// # Errors
///
/// This function errors if locking the engine mutex fails
pub fn get_stack() -> Vec<Bucket> {
    let engine = ENGINE.lock().unwrap();

    engine.stack.clone()
}

/// Get a list of valid commands that the engine accepts
pub fn get_commands() -> Vec<String> {
    COMMAND_MAPPINGS.keys().map(|s| s.to_owned()).collect()
}

/// Get the current previous answer from the engine.
///
/// # Errors
///
/// This function errors if locking the engine mutex fails
pub fn get_previous_answer() -> Bucket {
    let engine = ENGINE.lock().unwrap();

    engine.previous_answer.clone()
}

/// Update the previous answer variable in the engine.
///
/// This should be called after a full algebraic statement in algebraic mode,
/// or after each RPN command if in RPN mode.
///
/// # Errors
///
/// This function error if locking the engine mutex fails
pub fn update_previous_answer() -> EngineSignalSet {
    let mut engine = ENGINE.lock().unwrap();

    let result = engine.update_previous_answer();

    let mut signals = EngineSignalSet::new();
    signals.merge(result);

    signals
}
