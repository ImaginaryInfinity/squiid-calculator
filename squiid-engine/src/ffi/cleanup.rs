use std::ffi::{c_char, c_int, CString};

use super::data_structs::MessageActionSetFFI;

/// Free the error string contained within the MessageActionSetFFI struct
///
/// # Arguments
///
/// * `ptr` - Pointer to a MessageActionSetFFI struct which was returned from Rust
#[no_mangle]
extern "C" fn free_message_action_set(ptr: MessageActionSetFFI) {
    unsafe {
        if !ptr.error.is_null() {
            let _ = CString::from_raw(ptr.error);
            // the string will be automatically dropped after this
        }
    }
}

#[no_mangle]
extern "C" fn free_string_array(ptr: *mut *mut c_char, len: c_int) {
    let len = len as usize;

    // Get back our vector.
    // Previously we shrank to fit, so capacity == length.
    let v = unsafe { Vec::from_raw_parts(ptr, len, len) };

    // Now drop one string at a time.
    for elem in v {
        let s = unsafe { CString::from_raw(elem) };
        std::mem::drop(s);
    }

    // Afterwards the vector will be dropped and thus freed.
}
