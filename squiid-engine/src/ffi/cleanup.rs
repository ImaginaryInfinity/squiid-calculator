//! This module provides functions for freeing memory allocated for FFI (Foreign Function Interface) objects
//! These functions ensure that memory allocated for strings, arrays, and custom data structures
//! ([`EngineSignalSetFFI`], [`BucketFFI`], etc.) is properly deallocated when no longer needed.
//!
//! # Overview
//!
//! The Rust code interacting with foreign code (e.g., C) must manually manage memory
//! for objects returned over the FFI boundary. This module provides safe deallocation
//! functions to prevent memory leaks.
//!
//! # Functions
//!
//! - [`free_engine_signal_set`]: Frees an error string contained within an [`EngineSignalSetFFI`] struct.
//! - [`free_string_array`]: Frees an array of C strings (`char*`).
//! - [`free_bucket_array`]: Frees an array of [`BucketFFI`] objects.
//! - [`free_bucket`]: Frees a single [`BucketFFI`] object.
//!
//! # Safety Considerations
//!
//! - These functions must be called on objects that were allocated and returned from Rust.
//! - Calling these functions on invalid or already freed pointers will cause undefined behavior.
//! - Ensure that memory is properly managed across the FFI boundary to avoid double frees or leaks.

use std::ffi::{c_char, c_int, CString};

use super::data_structs::{BucketFFI, EngineSignalSetFFI};

/// Free the error string contained within the [`EngineSignalSetFFI`] struct
///
/// # Arguments
///
/// * `ptr` - Pointer to an [`EngineSignalSetFFI`] struct which was returned from Rust
#[unsafe(no_mangle)]
extern "C" fn free_engine_signal_set(ptr: EngineSignalSetFFI) {
    if !ptr.error.is_null() {
        free_string(ptr.error);
        // the string will be automatically dropped after this
    }
}

/// Free an array of strings that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `array` - the string array to free
/// * `len` - the length of the string array
///
/// # Panics
///
/// If the array pointer is null or if the vec or strings are invalid data
#[unsafe(no_mangle)]
extern "C" fn free_string_array(array: *mut *mut c_char, len: c_int) {
    if array.is_null() {
        return;
    }

    let len = len as usize;

    // Get back our vector.
    // Previously we shrank to fit, so capacity == length.
    let v = unsafe { Vec::from_raw_parts(array, len, len) };

    // Now drop one string at a time.
    for elem in v {
        free_string(elem);
    }

    // Afterwards the vector will be dropped and thus freed.
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

/// Free an array of Bucket objects that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `array` - the bucket array to free
/// * `len` - the length of the bucket array
///
/// # Panics
///
/// If the array pointer is null or if the vec or Bucket are invalid data
#[unsafe(no_mangle)]
extern "C" fn free_bucket_array(array: *mut *mut BucketFFI, len: c_int) {
    if array.is_null() {
        return;
    }

    let len = len as usize;

    // reconstruct vec
    // Previously we shrank to fit, so capacity == length.
    let array = unsafe { Vec::from_raw_parts(array, len, len) };

    for bucket_ffi in array {
        // iterate over each bucket and drop it
        free_bucket(bucket_ffi);
    }

    // vec to auto dropped here
}

/// Free a Bucket object that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `bucket_ffi` - The Bucket to free
///
/// # Panics
///
/// If the bucket pointer is null or if the bucket is invalid data
#[unsafe(no_mangle)]
extern "C" fn free_bucket(bucket_ffi: *mut BucketFFI) {
    if bucket_ffi.is_null() {
        return;
    }

    let bucket = unsafe { Box::from_raw(bucket_ffi) };

    // drop each bucket's string value
    if !bucket.value.is_null() {
        let s = unsafe { CString::from_raw(bucket.value) };
        std::mem::drop(s);
    }
}
