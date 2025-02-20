use std::{
    ffi::{c_char, CString},
    mem,
};

use super::ParseResultFFI;

/// Free an array of strings that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `parse_result` - the ParseResultFFI object that should be freed
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
