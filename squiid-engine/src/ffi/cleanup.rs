use std::ffi::{c_char, c_int, CString};

use super::data_structs::{BucketFFI, EngineSignalSetFFI};

/// Free the error string contained within the EngineSignalSetFFI struct
///
/// # Arguments
///
/// * `ptr` - Pointer to a EngineSignalSetFFI struct which was returned from Rust
#[no_mangle]
extern "C" fn free_engine_signal_set(ptr: EngineSignalSetFFI) {
    unsafe {
        if !ptr.error.is_null() {
            let _ = CString::from_raw(ptr.error);
            // the string will be automatically dropped after this
        }
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
#[no_mangle]
extern "C" fn free_bucket(bucket_ffi: *mut BucketFFI) {
    if bucket_ffi.is_null() {
        panic!("bucket pointer is null");
    }

    let bucket = unsafe { Box::from_raw(bucket_ffi) };

    // drop each bucket's string value
    if !bucket.value.is_null() {
        let s = unsafe { CString::from_raw(bucket.value) };
        std::mem::drop(s);
    }
}
