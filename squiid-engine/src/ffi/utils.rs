use std::ffi::{CStr, CString, c_char, c_int};

#[unsafe(no_mangle)]
pub extern "C" fn squiid_string_clone(s: *const c_char) -> *mut c_char {
    string_to_ffi(ffi_to_string(s))
}

pub fn string_to_ffi<S: AsRef<str>>(s: S) -> *mut c_char {
    CString::new(s.as_ref())
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

pub fn ffi_to_string(s: *const c_char) -> String {
    if s.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned()
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
///
/// WARN: we should eventually switch to c_size_t: https://github.com/rust-lang/rust/issues/88345
pub unsafe fn vec_to_ffi_array<T>(v: Vec<T>, out_len: *mut c_int) -> *mut T {
    let mut boxed_slice = v.into_boxed_slice();
    let len = boxed_slice.len();

    // leak the data so we can send it over ffi
    let ptr = boxed_slice.as_mut_ptr();
    std::mem::forget(boxed_slice);

    if !out_len.is_null() {
        unsafe { *out_len = len as c_int };
    }

    ptr
}

/// Reclaims a `Vec<T>` from a raw pointer and length in order to be cleaned up.
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
pub unsafe fn reclaim_ffi_array<T>(ptr: *mut T, len: c_int) -> Vec<T> {
    if ptr.is_null() || len == 0 {
        return Vec::new();
    }

    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr, len as usize);
        Box::from_raw(slice).into_vec()
    }
}
