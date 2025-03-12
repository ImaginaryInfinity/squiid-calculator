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

#![allow(clippy::mem_forget)]

use std::ffi::{c_char, c_int, CStr, CString};

use data_structs::{BucketFFI, EngineSignalSetFFI};

use crate::{execute_multiple_rpn, EngineSignalSet};

mod cleanup;
mod data_structs;
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
    let mut rpn_data_vec = Vec::new();

    // iterate over the submissions
    for i in 0..rpn_data_length {
        unsafe {
            // create new strings from the provided pointers and push them to the vec
            let c_str = CStr::from_ptr(*rpn_data.add(i));
            rpn_data_vec.push(match c_str.to_str() {
                Ok(str) => str,
                Err(e) => return EngineSignalSet::new().set_error(&e).into(),
            });
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
    let mut stack_ptr: Vec<*mut BucketFFI> = crate::get_stack()
        .iter()
        .map(|b| Box::into_raw(Box::new(BucketFFI::from(b.clone()))))
        .collect();

    stack_ptr.shrink_to_fit();
    // assert that shrink_to_fit worked
    assert!(stack_ptr.len() == stack_ptr.capacity());

    // write the vec length to the pointer that was passed in
    let len = stack_ptr.len();
    unsafe { std::ptr::write(outlen, len as c_int) };

    // get the pointer to the vec that we are returning
    let vec_ptr = stack_ptr.as_mut_ptr();
    std::mem::forget(stack_ptr);

    vec_ptr
}

/// Get the engine's list of currently supported commands.
///
/// # Arguments
///
/// * `outlen` - A pointer to an integer to store the length of the output array
#[unsafe(no_mangle)]
extern "C" fn get_commands_exposed(outlen: *mut c_int) -> *mut *mut c_char {
    // convert Vec of Strings into vec of raw pointers
    let mut commands: Vec<_> = crate::get_commands()
        .into_iter()
        .filter_map(|s| CString::new(s).ok().map(|c| c.into_raw()))
        .collect();

    if commands.len() != crate::get_commands().len() {
        unsafe { std::ptr::write(outlen, 0) };
        return std::ptr::null_mut();
    }

    // shrink capacity of vec
    commands.shrink_to_fit();
    assert!(commands.len() == commands.capacity());

    let len = commands.len();
    // forget pointer so that rust doesnt drop it
    let vec_ptr = commands.as_mut_ptr();
    std::mem::forget(commands);

    // write length to outlen
    unsafe { std::ptr::write(outlen, len as c_int) };

    vec_ptr
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
    let result = crate::update_previous_answer();

    EngineSignalSetFFI::from(result)
}
