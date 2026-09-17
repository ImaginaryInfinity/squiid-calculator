//! This module provides functions for freeing memory allocated for FFI (Foreign Function Interface) objects
//! These functions ensure that memory allocated for strings, arrays, and custom data structures
//! is properly deallocated when no longer needed.
//!
//! # Overview
//!
//! The Rust code interacting with foreign code (e.g., C) must manually manage memory
//! for objects returned over the FFI boundary. This module provides safe deallocation
//! functions to prevent memory leaks.
//!
//! # Functions
//!
//! - [`squiid_free_engine_signal_set`]: Frees an error string contained within an [`EngineSignalSetFFI`] struct.
//! - [`squiid_free_bucket_array`]: Frees an array of [`BucketFFI`] objects.
//! - [`squiid_free_bucket`]: Frees a single [`BucketFFI`] object.
//! - [`squiid_free_string_array`]: Frees an array of C strings (`char*`).
//!
//! # Safety Considerations
//!
//! - These functions must be called on objects that were allocated and returned from Rust.
//! - Calling these functions on invalid or already freed pointers will cause undefined behavior.
//! - Ensure that memory is properly managed across the FFI boundary to avoid double frees or leaks.

use std::ffi::{CString, c_char, c_int};

use crate::ffi::{data_structs::EngineSignalSetFFI, utils::reclaim_ffi_array};

use super::data_structs::BucketFFI;

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
extern "C" fn squiid_free_bucket_array(array: *mut BucketFFI, len: c_int) {
    let v = unsafe { reclaim_ffi_array(array, len) };
    for bucket_ffi in v.iter() {
        squiid_free_string(bucket_ffi.value);
    }
}

/// Free a Bucket object that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `bucket_ffi` - The Bucket to free
#[unsafe(no_mangle)]
extern "C" fn squiid_free_bucket(bucket_ffi: *mut BucketFFI) {
    if !bucket_ffi.is_null() {
        let bucket = unsafe { Box::from_raw(bucket_ffi) };

        // drop the bucket's string value
        squiid_free_string(bucket.value);
    }
}

/// Free the error string contained within the [`EngineSignalSetFFI`] struct
///
/// # Arguments
///
/// * `ptr` - Pointer to an [`EngineSignalSetFFI`] struct which was returned from Rust
#[unsafe(no_mangle)]
extern "C" fn squiid_free_engine_signal_set(ptr: EngineSignalSetFFI) {
    if !ptr.error.is_null() {
        squiid_free_string(ptr.error);
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
extern "C" fn squiid_free_string_array(array: *mut *mut c_char, len: c_int) {
    unsafe {
        let v = reclaim_ffi_array(array, len);
        for elem in v.iter() {
            squiid_free_string(*elem);
        }

        // Afterwards the vector will be dropped and thus freed.
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn squiid_free_string(s: *mut c_char) {
    if !s.is_null() {
        std::mem::drop(unsafe { CString::from_raw(s) });
    }
}
