use std::ffi::{c_char, c_void, CString};

use crate::ffi::config_manager::to_cstring;

/// FFI-Compatible String Result type
#[repr(C)]
pub struct FFIResult {
    /// Whether this is ok or err
    pub ok: bool,
    /// The value
    pub value: *mut c_void,
    /// The error message
    pub error: *mut c_char,
}

impl FFIResult {
    pub fn ok<V>(v: V) -> Self {
        let boxed = Box::new(v);
        let ptr = Box::into_raw(boxed);
        Self {
            ok: true,
            value: ptr as *mut c_void,
            error: std::ptr::null_mut(),
        }
    }

    pub fn err(e: impl AsRef<str>) -> Self {
        Self {
            ok: false,
            value: std::ptr::null_mut(),
            error: to_cstring(e.as_ref()),
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
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
                string_val: to_cstring(s),
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
                string_val: to_cstring(datetime.to_string()),
                ..Default::default()
            },
            toml::Value::Array(values) => {
                let arr: Vec<FFIValue> = values.into_iter().map(Self::from).collect();
                let len = arr.len();
                let slice = arr.into_boxed_slice();
                let ptr = Box::into_raw(slice) as *mut FFIValue;

                Self {
                    kind: FFIValueKind::Array,
                    array: ptr,
                    array_len: len,
                    ..Default::default()
                }
            }
            toml::Value::Table(map) => {
                let mut keys = Vec::with_capacity(map.len());
                let mut vals = Vec::with_capacity(map.len());

                for (k, v) in map {
                    keys.push(to_cstring(k.clone()));
                    vals.push(Self::from(v));
                }

                let len = keys.len();
                let keys_ptr = Box::into_raw(keys.into_boxed_slice()) as *mut *mut c_char;
                let vals_ptr = Box::into_raw(vals.into_boxed_slice()) as *mut FFIValue;

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

impl Drop for FFIValue {
    fn drop(&mut self) {
        unsafe {
            match self.kind {
                FFIValueKind::String | FFIValueKind::Datetime => {
                    if !self.string_val.is_null() {
                        drop(CString::from_raw(self.string_val));
                    }
                }
                FFIValueKind::Array => {
                    if !self.array.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.array, self.array_len);
                        drop(Box::from_raw(slice));
                    }
                }
                FFIValueKind::Table => {
                    if !self.table_vals.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.table_keys, self.table_len);
                        let keys = Box::from_raw(slice);
                        for k in keys.iter() {
                            if !k.is_null() {
                                drop(CString::from_raw(*k));
                            }
                        }
                    }

                    if !self.table_keys.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.table_vals, self.table_len);
                        drop(Box::from_raw(slice));
                    }
                }
                FFIValueKind::Integer | FFIValueKind::Float | FFIValueKind::Boolean => (),
            }
        }
    }
}

macro_rules! free_ffi_result {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            #[doc="Free an [`FFIResult`] object containing a `" $name "` that was returned over the FFI boundary."]
            #[doc=""]
            #[doc="# Arguments"]
            #[doc=""]
            #[doc="* `ffi_result` - The [`FFIResult`] to free"]
            #[unsafe(no_mangle)]
            extern "C" fn [<free_ffi_ $name _result>](ffi_result: *mut FFIResult) {
                if !ffi_result.is_null() {
                    let result = unsafe { Box::from_raw(ffi_result) };
                    if !result.value.is_null() {
                        drop(unsafe { Box::from_raw(result.value as $ty) });
                    }
                    if !result.error.is_null() {
                        drop(unsafe { CString::from_raw(result.error) });
                    }
                }
            }
        }
    };
}

free_ffi_result!(String, *mut c_char);
free_ffi_result!(FFIValue, *mut FFIValue);
