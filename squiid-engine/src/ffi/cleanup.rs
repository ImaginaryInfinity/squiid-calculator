use std::ffi::{c_char, c_int, CString};

use super::data_structs::{BucketFFI, MessageActionSetFFI};

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
extern "C" fn free_string_array(array: *mut *mut c_char, len: c_int) {
    if array.is_null() {
        panic!("array pointer is null")
    }

    let len = len as usize;

    // Get back our vector.
    // Previously we shrank to fit, so capacity == length.
    let v = unsafe { Vec::from_raw_parts(array, len, len) };

    // Now drop one string at a time.
    for elem in v {
        let s = unsafe { CString::from_raw(elem) };
        std::mem::drop(s);
    }

    // Afterwards the vector will be dropped and thus freed.
}

#[no_mangle]
extern "C" fn free_bucket_array(array: *mut *mut BucketFFI, len: c_int) {
    if array.is_null() {
        panic!("array pointer is null");
    }

    let len = len as usize;

    // reconstruct vec
    // Previously we shrank to fit, so capacity == length.
    let array = unsafe { Vec::from_raw_parts(array, len, len) };

    for bucket_ffi in array {
        // iterate over each bucket and get it back
        if !bucket_ffi.is_null() {
            let bucket = unsafe { Box::from_raw(bucket_ffi) };

            // drop each bucket's string value
            if !bucket.value.is_null() {
                let s = unsafe { CString::from_raw(bucket.value) };
                std::mem::drop(s);
            }
        }
    }

    // vec to auto dropped here
}
