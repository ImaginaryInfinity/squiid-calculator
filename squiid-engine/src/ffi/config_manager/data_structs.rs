use std::ffi::{c_char, CString};

use crate::ffi::config_manager::to_cstring;

/// FFI-Compatible String Result type
#[repr(C)]
pub struct FFIStringResult {
    /// Whether this is ok or err
    pub ok: bool,
    /// The value
    pub value: *mut c_char,
    /// The error message
    pub error: *mut c_char,
}

impl FFIStringResult {
    pub fn ok(v: impl AsRef<str>) -> Self {
        Self {
            ok: true,
            value: to_cstring!(v.as_ref()),
            error: std::ptr::null_mut(),
        }
    }

    pub fn err(e: impl AsRef<str>) -> Self {
        Self {
            ok: false,
            value: std::ptr::null_mut(),
            error: to_cstring!(e.as_ref()),
        }
    }
}

/// FFI-Compatible [`FFIValue`] Result type
#[repr(C)]
pub struct FFIValueResult {
    /// Whether this is ok or err
    pub ok: bool,
    /// The value
    pub value: FFIValue,
    /// The error message
    pub error: *mut c_char,
}

impl FFIValueResult {
    pub fn ok(v: FFIValue) -> Self {
        Self {
            ok: true,
            value: v,
            error: std::ptr::null_mut(),
        }
    }

    pub fn err(e: impl AsRef<str>) -> Self {
        Self {
            ok: false,
            value: FFIValue::default(),
            error: to_cstring!(e.as_ref()),
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub enum FFIValueKind {
    #[default]
    String,
    Integer,
    Float,
    Boolean,
    Datetime,
    Array,
    Table,
}

#[repr(C)]
#[derive(Default)]
pub struct FFIValue {
    pub kind: FFIValueKind,
    pub string_val: *mut c_char,
    pub int_val: i64,
    pub float_val: f64,
    pub bool_val: bool,
    pub array: *mut FFIValue,
    pub array_len: usize,
    pub table_keys: *mut *mut c_char,
    pub table_vals: *mut FFIValue,
    pub table_len: usize,
}

impl From<toml::Value> for FFIValue {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) => Self {
                kind: FFIValueKind::String,
                string_val: to_cstring!(s),
                ..Default::default()
            },
            toml::Value::Integer(i) => Self {
                kind: FFIValueKind::Integer,
                int_val: i,
                ..Default::default()
            },
            toml::Value::Float(f) => Self {
                kind: FFIValueKind::Float,
                float_val: f,
                ..Default::default()
            },
            toml::Value::Boolean(b) => Self {
                kind: FFIValueKind::Boolean,
                bool_val: b,
                ..Default::default()
            },
            toml::Value::Datetime(datetime) => Self {
                kind: FFIValueKind::Datetime,
                string_val: to_cstring!(datetime.to_string()),
                ..Default::default()
            },
            toml::Value::Array(values) => {
                let mut arr: Vec<FFIValue> = values.into_iter().map(Self::from).collect();
                let ptr = arr.as_mut_ptr();
                let len = arr.len();
                std::mem::forget(arr);

                Self {
                    kind: FFIValueKind::Array,
                    array: ptr,
                    array_len: len,
                    ..Default::default()
                }
            }
            toml::Value::Table(map) => {
                let mut keys: Vec<*mut c_char> = Vec::new();
                let mut vals: Vec<FFIValue> = Vec::new();

                for (k, v) in map {
                    keys.push(to_cstring!(k.clone()));
                    vals.push(Self::from(v));
                }

                let keys_ptr = keys.as_mut_ptr();
                let vals_ptr = vals.as_mut_ptr();
                let len = keys.len();

                std::mem::forget(keys);
                std::mem::forget(vals);

                Self {
                    kind: FFIValueKind::Table,
                    table_keys: keys_ptr,
                    table_vals: vals_ptr,
                    table_len: len,
                    ..Default::default()
                }
            }
        }
    }
}

/// Free an [`FFIStringResult`] object that was returned over the FFI boundary.
///
/// # Arguments
///
/// * `ffi_result` - The [`FFIStringResult`] to free
#[unsafe(no_mangle)]
extern "C" fn free_ffi_string_result(ffi_result: *mut FFIStringResult) {
    if !ffi_result.is_null() {
        let result = unsafe { Box::from_raw(ffi_result) };
        if !result.value.is_null() {
            std::mem::drop(unsafe { CString::from_raw(result.value) });
        }
        if !result.error.is_null() {
            std::mem::drop(unsafe { CString::from_raw(result.error) });
        }
    }
}
