use std::ffi::c_char;

use crate::ffi::{cleanup::squiid_free_string, utils::string_to_ffi};

mod ffi_value;
pub use ffi_value::{FFIValue, FFIValueKind, ffi_value_free};

/// FFI-Compatible String Result type
#[repr(C)]
pub struct FFIResult {
    /// Whether this is ok or err
    pub ok: bool,
    /// The value
    pub value: *mut FFIValue,
    /// The error message
    pub error: *mut c_char,
}

impl FFIResult {
    pub fn ok(v: FFIValue) -> Self {
        let boxed = Box::new(v);
        let ptr = Box::into_raw(boxed);
        Self {
            ok: true,
            value: ptr,
            error: std::ptr::null_mut(),
        }
    }

    pub fn err(e: impl AsRef<str>) -> Self {
        Self {
            ok: false,
            value: std::ptr::null_mut(),
            error: string_to_ffi(e.as_ref()),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_result_free(ptr: *mut FFIResult) {
    if !ptr.is_null() {
        let res = unsafe { Box::from_raw(ptr) };
        ffi_value_free(res.value);
        squiid_free_string(res.error);
    }
}

impl<E> Into<FFIResult> for Result<String, E>
where
    E: ToString,
{
    fn into(self) -> FFIResult {
        match self {
            Ok(v) => FFIResult::ok(v.into()),
            Err(e) => FFIResult::err(e.to_string()),
        }
    }
}

impl<E> Into<FFIResult> for Result<(), E>
where
    E: ToString,
{
    fn into(self) -> FFIResult {
        match self {
            Ok(_) => FFIResult::ok("".into()),
            Err(e) => FFIResult::err(e.to_string()),
        }
    }
}

impl<E> Into<FFIResult> for Result<toml::Value, E>
where
    E: ToString,
{
    fn into(self) -> FFIResult {
        match self {
            Ok(v) => FFIResult::ok(FFIValue::from(v)),
            Err(e) => FFIResult::err(e.to_string()),
        }
    }
}
