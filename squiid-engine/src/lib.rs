//! # Squiid Engine
//!
//! Squiid Engine provides a reverse Polish notation (RPN) calculation engine, along with utilities
//! for managing commands, processing input, and handling execution signals.
//!
//! ## Modules
//!
//! - [`bucket`]: Defines the [`Bucket`] type used for storing values in the engine.
//! - [`command_mappings`]: Contains the mapping of commands to their respective functions.
//! - [`engine`]: Implements the core RPN engine.
//! - [`crash_reporter`] *(optional)*: Handles crash reporting when the `crash-reporting` feature is enabled.
//!
//! ## Global Structures
//!
//! - `ENGINE`: A globally accessible instance of the RPN engine.
//! - `COMMAND_MAPPINGS`: A lookup table mapping commands to engine operations.
//!
//! ## Core Functionality
//!
//! - [`handle_data`]: Processes a single RPN command or numeric input.
//! - [`execute_multiple_rpn`]: Executes a series of RPN commands in sequence.
//! - [`get_stack`]: Retrieves the current stack state.
//! - [`get_commands`]: Returns a list of valid commands.
//! - [`get_previous_answer`]: Fetches the last computed result.
//! - [`update_previous_answer`]: Updates the stored previous answer in the engine.
//!
//! # Example Usage
//!
//! ```rust
//! use squiid_engine::execute_multiple_rpn;
//!
//! let result = execute_multiple_rpn(vec!["5", "3", "+"]);
//! assert!(result.stack_updated());
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::missing_panics_doc)]

pub mod bucket;
pub mod command_mappings;
pub mod engine;
mod utils;

#[cfg(feature = "crash-reporting")]
pub mod crash_reporter;

#[cfg(feature = "ffi")]
mod ffi;

use std::{
    borrow::BorrowMut,
    sync::{LazyLock, Mutex},
};

use bucket::Bucket;
use command_mappings::CommandsMap;
use engine::Engine;

/// The global engine struct used for processing calculations
static ENGINE: LazyLock<Mutex<Engine>> = LazyLock::new(|| Mutex::new(Engine::new()));
/// The mapping of commands to functions in the [`Engine`]
static COMMAND_MAPPINGS: LazyLock<CommandsMap> =
    LazyLock::new(command_mappings::create_function_map);

/// Represents the different signals that can be returned by the engine.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum EngineSignal {
    /// The stack was updated
    StackUpdated,
    /// Signals a request to terminate execution.
    Quit,
    /// No operation
    NOP,
}

/// Processes a single RPN command or numeric input.
///
/// # Arguments
///
/// * `engine` - A mutable reference to the engine instance.
/// * `data` - The command or number to be executed.
///
/// # Errors
///
/// Returns an error if an invalid command is executed or an undefined variable is referenced.
///
/// # Behavior
///
/// - Maintains an undo history for up to 20 operations.
/// - Ignores history updates for "refresh", "undo", and "redo" commands.
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

    match COMMAND_MAPPINGS.get(data) {
        Some(func) => func(engine.borrow_mut()),
        None => {
            // return result value of adding item to stack
            engine.add_item_to_stack(Bucket::from(data.to_string()))
        }
    }
}

/// Struct to identify which [`EngineSignal`]s were triggered during the submission of multiple
/// commands to the engine (usually in `execute_rpn_data`)
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct EngineSignalSet {
    /// This is set if the `get_stack` method should be called to retrieve the new stack
    stack_updated: bool,
    /// This is set if the frontend should quit
    quit: bool,
    /// This is set if there was an error while putting data into the engine
    error: Option<String>,
}

impl EngineSignalSet {
    /// Creates a new [`EngineSignalSet`] with default values.
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

    /// Set an error in the signal set and return a new [`EngineSignalSet`]
    pub fn set_error(&mut self, error: &(impl ToString + ?Sized)) -> Self {
        self.error = Some(error.to_string());
        self.clone()
    }

    /// Returns `true` if the stack was updated.
    pub fn stack_updated(&self) -> bool {
        self.stack_updated
    }

    /// Returns `true` if a quit signal was triggered.
    pub fn should_quit(&self) -> bool {
        self.quit
    }

    /// Get the last encountered error, if available
    pub fn get_error(&self) -> Option<String> {
        self.error.clone()
    }
}

/// Execute multiple RPN commands in the engine sequentially.
///
/// # Arguments
///
/// * `rpn_data` - A vector of RPN commands to execute
///
/// # Returns
///
/// An [`EngineSignalSet`] indicating which actions occurred during execution.
pub fn execute_multiple_rpn(rpn_data: Vec<&str>) -> EngineSignalSet {
    let Ok(mut engine) = ENGINE.lock() else {
        return EngineSignalSet::new().set_error("unable to lock engine mutex");
    };

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

/// Executes a single RPN statement
#[macro_export]
macro_rules! execute_single_rpn {
    ($i:expr) => {
        execute_multiple_rpn(vec![$i])
    };
}

/// Get the current stack from the engine.
///
/// # Panics
///
/// Panics if the engine mutex cannot be locked.
#[allow(clippy::expect_used)]
pub fn get_stack() -> Vec<Bucket> {
    let engine = ENGINE.lock().expect("engine mutex is poisoned");

    engine.stack.clone()
}

/// Get a list of valid commands that the engine accepts
pub fn get_commands() -> Vec<String> {
    COMMAND_MAPPINGS.keys().map(|s| s.to_owned()).collect()
}

/// Get the current previous answer from the engine.
///
/// # Panics
///
/// Panics if the engine mutex cannot be locked.
#[allow(clippy::expect_used)]
pub fn get_previous_answer() -> Bucket {
    let engine = ENGINE.lock().expect("engine mutex is poisoned");

    engine.previous_answer.clone()
}

/// Update the previous answer variable in the engine.
///
/// This should be called after a full algebraic statement in algebraic mode,
/// or after each RPN command if in RPN mode.
pub fn update_previous_answer() -> EngineSignalSet {
    let Ok(mut engine) = ENGINE.lock() else {
        return EngineSignalSet::new().set_error("unable to lock engine mutex");
    };

    let result = engine.update_previous_answer();

    let mut signals = EngineSignalSet::new();
    signals.merge(result);

    signals
}
