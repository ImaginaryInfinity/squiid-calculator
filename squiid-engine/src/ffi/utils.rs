use std::ffi::{c_char, c_int, CString};

pub fn to_cstring<S: Into<Vec<u8>>>(s: S) -> *mut c_char {
    match CString::new(s) {
        Ok(v) => v.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Convert a Vec<T> into an FFI-compatible C array and writes the length to `out_len`.
///
/// # Arguments
///
/// * `v` - The vector
/// * `out_len` - The c_int where the length of the array is written
///
/// # Safety
///
/// This function leaks memory which must be freed using `reclaim_ffi_array`.
pub unsafe fn vec_to_ffi_array<T>(v: Vec<T>, out_len: *mut c_int) -> *mut T {
    let boxed_slice = v.into_boxed_slice();
    let len = boxed_slice.len();

    // leak the data so we can send it over ffi
    let ptr = Box::into_raw(boxed_slice) as *mut T;

    if !out_len.is_null() {
        unsafe { *out_len = len as c_int };
    }

    ptr
}

/// Reclaims a `Box<[T]>` from a raw pointer and length in order to be cleaned up.
///
/// # Arguments
///
/// * `ptr` - The pointer to the array
/// * `len` - The length of the Array
///
/// # Safety
///
/// `ptr` must have been created by `vec_to_ffi_array` and `len` must match the real length of the
/// array.
pub unsafe fn reclaim_ffi_array<T>(ptr: *mut T, len: c_int) -> Box<[T]> {
    if ptr.is_null() {
        return Box::new([]);
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize);
        Box::from_raw(slice)
    }
}
