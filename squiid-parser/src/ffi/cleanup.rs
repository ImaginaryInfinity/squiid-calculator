//! This module provides functions for freeing memory allocated for FFI (Foreign Function Interface) objects
//! These functions ensure that memory allocated for strings, arrays, and custom data structures
//! ([`ParseResultFFI`], etc.) is properly deallocated when no longer needed.
//!
//! # Overview
//!
//! The Rust code interacting with foreign code (e.g., C) must manually manage memory
//! for objects returned over the FFI boundary. This module provides safe deallocation
//! functions to prevent memory leaks.
//!
//! # Functions
//!
//! - [`free_parse_result`]: Frees strings and/or an error string contained within a [`ParseResultFFI`] struct.
//! - [`free_string`]: Frees an array of C strings (`char*`).
//!
//! # Safety Considerations
//!
//! - These functions must be called on objects that were allocated and returned from Rust.
//! - Calling these functions on invalid or already freed pointers will cause undefined behavior.
//! - Ensure that memory is properly managed across the FFI boundary to avoid double frees or leaks.

use std::{
    ffi::{c_char, CString},
    mem,
};

use super::ParseResultFFI;

/// Free an array of strings that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `parse_result` - the [`ParseResultFFI`] object that should be freed
///
/// # Panics
///
/// If the strings in the vec are invalid data
#[unsafe(no_mangle)]
extern "C" fn free_parse_result(parse_result: ParseResultFFI) {
    let len = parse_result.result_len as usize;

    if !parse_result.result.is_null() {
        // Get back our vector.
        // Previously we shrank to fit, so capacity == length.
        let v = unsafe { Vec::from_raw_parts(parse_result.result, len, len) };

        // Now drop one string at a time.
        for elem in v {
            let s = unsafe { CString::from_raw(elem) };
            mem::drop(s);
        }

        // Afterwards the vector will be dropped and thus freed.
    }

    // Free the error string
    if !parse_result.error.is_null() {
        let _ = unsafe { CString::from_raw(parse_result.error) };
    }
}

/// Free a string that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `string` - the string to free
#[unsafe(no_mangle)]
extern "C" fn free_string(string: *mut c_char) {
    if string.is_null() {
        return;
    }

    unsafe {
        std::mem::drop(CString::from_raw(string));
    }
}
