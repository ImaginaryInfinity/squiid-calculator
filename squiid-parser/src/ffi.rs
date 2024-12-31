use std::{
    ffi::{CStr, CString},
    mem,
    os::raw::{c_char, c_int},
    ptr,
};

use crate::parse;

/// Parse a given algebraic (infix) notation string into an array of RPN (postfix) commands.
///
/// # Arguments
///
/// * `input` - The string input to parse
/// * `outlen` - A pointer to an integer to store the length of the result array
#[no_mangle]
extern "C" fn parse_exposed(input: *const c_char, outlen: *mut c_int) -> *mut *mut c_char {
    let c_str = unsafe { CStr::from_ptr(input) };
    let input_string = c_str.to_str().expect("Invalid UTF-8 string");

    let parsed_input = parse(input_string).unwrap();

    // Convert parsed input to Vec<CString>
    let c_strings: Vec<CString> = parsed_input
        .into_iter()
        .map(|s| CString::new(s).unwrap())
        .collect();

    // Turning each null-terminated string into a pointer.
    // `into_raw` takes ownershop, gives us the pointer and does NOT drop the data.
    let mut out = c_strings
        .into_iter()
        .map(|s| s.into_raw())
        .collect::<Vec<_>>();

    out.shrink_to_fit();
    assert!(out.len() == out.capacity());

    // get the pointer to the vector
    let len = out.len();
    let vec_ptr = out.as_mut_ptr();
    mem::forget(out);

    unsafe { ptr::write(outlen, len as c_int) };

    vec_ptr
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
#[no_mangle]
extern "C" fn free_string_array(array: *mut *mut c_char, len: c_int) {
    let len = len as usize;

    // Get back our vector.
    // Previously we shrank to fit, so capacity == length.
    let v = unsafe { Vec::from_raw_parts(array, len, len) };

    // Now drop one string at a time.
    for elem in v {
        let s = unsafe { CString::from_raw(elem) };
        mem::drop(s);
    }

    // Afterwards the vector will be dropped and thus freed.
}
