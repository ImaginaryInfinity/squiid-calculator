//! Foreign Function Interface (FFI) bindings for Squiid engine.
//!
//! This module provides FFI-exposed functions to interact with the engine from external code,
//! such as C or other languages that support C-style linking. It allows submitting RPN commands,
//! retrieving the stack, fetching available commands, and managing the engine’s state.
//!
//! # Safety
//!
//! These functions cross the FFI boundary, meaning they deal with raw pointers and manual memory management.
//! Callers must ensure proper handling of allocated memory and adhere to Rust's ownership model to prevent
//! undefined behavior.
//!
//! # Exposed Functions
//!
//! - [`execute_multiple_rpn_exposed`] - Submits multiple RPN commands to the engine.
//! - [`get_stack_exposed`] - Retrieves the engine’s current stack.
//! - [`get_commands_exposed`] - Returns the list of supported commands.
//! - [`get_previous_answer_exposed`] - Fetches the last computed result.
//! - [`update_previous_answer_exposed`] - Updates the previous answer in the engine.
//!
//! # Modules
//!
//! - [`cleanup`] - Handles memory cleanup for FFI-exposed data.
//! - [`data_structs`] - Defines FFI-compatible data structures for interacting with the engine.
//!
//! # Usage
//!
//! These functions are primarily intended for use in external applications interfacing with the engine
//! via C bindings. Care should be taken when passing and handling pointers, as improper usage may
//! lead to memory leaks or undefined behavior.

use std::ffi::{CStr, c_char, c_int};

use data_structs::{BucketFFI, EngineSignalSetFFI};

use crate::{
    EngineSignalSet, execute_multiple_rpn,
    ffi::utils::{to_cstring, vec_to_ffi_array},
};

mod cleanup;
mod config_manager;
mod data_structs;
mod utils;
mod version;

/// FFI-Exposed function to submit multiple RPN commands to the engine.
///
/// # Arguments
///
/// * `rpn_data` - the RPN data array of strings to execute
/// * `rpn_data_length` - the length of `rpn_data`
///
/// # Safety
///
/// This function is unsafe because it is exposed over the FFI boundary. It dereferences a pointer
/// to access the `rpn_data` array
#[unsafe(no_mangle)]
extern "C" fn execute_multiple_rpn_exposed(
    rpn_data: *const *const c_char,
    rpn_data_length: usize,
) -> EngineSignalSetFFI {
    // construct a new vec to hold the data send from the frontend
    let mut rpn_data_vec = Vec::with_capacity(rpn_data_length);

    // iterate over the submissions
    for i in 0..rpn_data_length {
        unsafe {
            // create new strings from the provided pointers and push them to the vec
            let p = *rpn_data.add(i);
            if p.is_null() {
                return EngineSignalSet::new()
                    .set_error("Received null pointer in command array")
                    .into();
            }

            let c_str = CStr::from_ptr(p);
            match c_str.to_str() {
                Ok(str) => rpn_data_vec.push(str),
                Err(e) => return EngineSignalSet::new().set_error(&e).into(),
            };
        }
    }

    // submit all of the commands to the engine
    let result = execute_multiple_rpn(rpn_data_vec);

    // return a struct telling the frontend what to do next
    result.into()
}

/// Get the engine's current stack.
///
/// # Arguments
///
/// * `outlen` - A pointer to an integer to store the length of the output array
#[unsafe(no_mangle)]
extern "C" fn get_stack_exposed(outlen: *mut c_int) -> *mut *mut BucketFFI {
    // Create a vector of CStrings from the stack
    let stack_ptr: Vec<*mut BucketFFI> = crate::get_stack()
        .iter()
        .map(|b| Box::into_raw(Box::new(BucketFFI::from(b.clone()))))
        .collect();

    unsafe { vec_to_ffi_array(stack_ptr, outlen) }
}

/// Get the engine's list of currently supported commands.
///
/// # Arguments
///
/// * `outlen` - A pointer to an integer to store the length of the output array
#[unsafe(no_mangle)]
extern "C" fn get_commands_exposed(outlen: *mut c_int) -> *mut *mut c_char {
    // convert Vec of Strings into vec of raw pointers
    let commands: Vec<*mut c_char> = crate::get_commands()
        .into_iter()
        .map(|s| to_cstring(s))
        .collect();

    unsafe { vec_to_ffi_array(commands, outlen) }
}

/// Get the current previous answer from the engine.
#[unsafe(no_mangle)]
extern "C" fn get_previous_answer_exposed() -> *mut BucketFFI {
    Box::into_raw(Box::new(BucketFFI::from(crate::get_previous_answer())))
}

/// Update the previous answer variable in the engine.
///
/// This should be called after a full algebraic statement in algebraic mode,
/// or after each RPN command if in RPN mode.
#[unsafe(no_mangle)]
extern "C" fn update_previous_answer_exposed() -> EngineSignalSetFFI {
    EngineSignalSetFFI::from(crate::update_previous_answer())
}
