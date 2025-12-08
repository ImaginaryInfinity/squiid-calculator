use std::ffi::{c_char, c_int, CStr, CString};

use crate::{
    config_handler::ConfigBackend,
    ffi::{
        config_manager::data_structs::{FFIResult, FFIValue, FFIValueKind},
        utils::{to_cstring, vec_to_ffi_array},
    },
};

mod data_structs;

macro_rules! cstr_arg {
    ($ptr:expr, FFIResult) => {{
        if $ptr.is_null() {
            return $crate::ffi::config_manager::data_structs::FFIResult::err(
                "unexpected null parameter",
            );
        }

        unsafe {
            match std::ffi::CStr::from_ptr($ptr).to_str() {
                Ok(s) => s,
                Err(e) => {
                    return $crate::ffi::config_manager::data_structs::FFIResult::err(
                        e.to_string(),
                    );
                }
            }
        }
    }};

    ($ptr:expr, bool) => {{
        if $ptr.is_null() {
            return false;
        }

        unsafe {
            match std::ffi::CStr::from_ptr($ptr).to_str() {
                Ok(s) => s,
                Err(_) => {
                    return false;
                }
            }
        }
    }};
}

impl<E> Into<FFIResult> for Result<String, E>
where
    E: ToString,
{
    fn into(self) -> FFIResult {
        match self {
            Ok(v) => FFIResult::ok(v),
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
            Ok(_) => FFIResult::ok(""),
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

impl TryFrom<FFIValue> for toml::Value {
    type Error = String;

    fn try_from(value: FFIValue) -> Result<Self, Self::Error> {
        match value.kind {
            FFIValueKind::String => {
                if value.string_val.is_null() {
                    return Err("string variant of FFIValue::String mustn't be null".to_owned());
                }

                match unsafe { CStr::from_ptr(value.string_val) }.to_str() {
                    Ok(v) => Ok(toml::Value::String(v.to_owned())),
                    Err(e) => Err(e.to_string()),
                }
            }
            FFIValueKind::Integer => Ok(toml::Value::Integer(value.int_val)),
            FFIValueKind::Float => Ok(toml::Value::Float(value.float_val)),
            FFIValueKind::Boolean => Ok(toml::Value::Boolean(value.bool_val)),
            FFIValueKind::Datetime => {
                if value.string_val.is_null() {
                    return Err("string variant of FFIValue::String mustn't be null".to_owned());
                }

                let str = match unsafe { CStr::from_ptr(value.string_val) }.to_str() {
                    Ok(v) => v.to_owned(),
                    Err(e) => return Err(e.to_string()),
                };

                let dt: toml::value::Datetime = str
                    .parse()
                    .map_err(|e: toml::value::DatetimeParseError| e.to_string())?;
                Ok(toml::Value::Datetime(dt))
            }
            FFIValueKind::Array => {
                if value.array.is_null() {
                    if value.array_len == 0 {
                        return Ok(toml::Value::Array(Vec::new()));
                    }

                    return Err("Array has null pointer and non-zero length".to_owned());
                }

                let slice = unsafe { std::slice::from_raw_parts(value.array, value.array_len) };
                let mut result = Vec::with_capacity(value.array_len);

                for (i, item) in slice.iter().enumerate() {
                    let cloned = unsafe { clone_ffi_value(item)? };
                    result.push(
                        cloned
                            .try_into()
                            .map_err(|e| format!("array element {}: {}", i, e))?,
                    );
                }

                Ok(toml::Value::Array(result))
            }
            FFIValueKind::Table => {
                if value.table_keys.is_null() || value.table_vals.is_null() {
                    if value.table_len == 0 {
                        return Ok(toml::Value::Table(toml::map::Map::new()));
                    }
                    return Err("Table has null pointer and non-zero length".to_owned());
                }

                let keys_slice =
                    unsafe { std::slice::from_raw_parts(value.table_keys, value.table_len) };
                let vals_slice =
                    unsafe { std::slice::from_raw_parts(value.table_vals, value.table_len) };

                let mut result = toml::map::Map::new();
                for i in 0..value.table_len {
                    if keys_slice[i].is_null() {
                        return Err(format!("Table key {i} is null"));
                    }

                    let key = unsafe { CStr::from_ptr(keys_slice[i]) }
                        .to_str()
                        .map_err(|e| e.to_string())?
                        .to_string();

                    let cloned = unsafe { clone_ffi_value(&vals_slice[i])? };
                    let val = cloned
                        .try_into()
                        .map_err(|e| format!("Table value for key {key}: {e}"))?;
                    result.insert(key, val);
                }

                Ok(toml::Value::Table(result))
            }
        }
    }
}

unsafe fn clone_ffi_value(value: &FFIValue) -> Result<FFIValue, String> {
    match value.kind {
        FFIValueKind::String | FFIValueKind::Datetime => {
            if value.string_val.is_null() {
                return Ok(FFIValue {
                    kind: value.kind,
                    string_val: std::ptr::null_mut(),
                    ..Default::default()
                });
            }

            let str = unsafe {
                CStr::from_ptr(value.string_val)
                    .to_str()
                    .map_err(|e| e.to_string())?
                    .to_string()
            };
            Ok(FFIValue {
                kind: value.kind,
                string_val: to_cstring(str),
                ..Default::default()
            })
        }
        FFIValueKind::Integer => Ok(FFIValue {
            kind: value.kind,
            int_val: value.int_val,
            ..Default::default()
        }),
        FFIValueKind::Float => Ok(FFIValue {
            kind: value.kind,
            float_val: value.float_val,
            ..Default::default()
        }),
        FFIValueKind::Boolean => Ok(FFIValue {
            kind: value.kind,
            bool_val: value.bool_val,
            ..Default::default()
        }),
        FFIValueKind::Array => {
            if value.array.is_null() {
                return Ok(FFIValue {
                    kind: value.kind,
                    array: std::ptr::null_mut(),
                    array_len: 0,
                    ..Default::default()
                });
            }

            let slice = unsafe { std::slice::from_raw_parts(value.array, value.array_len) };
            let mut cloned = Vec::with_capacity(value.array_len);

            for item in slice.iter() {
                cloned.push(unsafe { clone_ffi_value(item)? });
            }

            let len = cloned.len();
            let ptr = Box::into_raw(cloned.into_boxed_slice()) as *mut FFIValue;
            Ok(FFIValue {
                kind: value.kind,
                array: ptr,
                array_len: len,
                ..Default::default()
            })
        }
        FFIValueKind::Table => {
            if value.table_keys.is_null() || value.table_vals.is_null() {
                return Ok(FFIValue {
                    kind: value.kind,
                    table_keys: std::ptr::null_mut(),
                    table_vals: std::ptr::null_mut(),
                    table_len: 0,
                    ..Default::default()
                });
            }

            let keys_slice =
                unsafe { std::slice::from_raw_parts(value.table_keys, value.table_len) };
            let vals_slice =
                unsafe { std::slice::from_raw_parts(value.table_vals, value.table_len) };

            let mut cloned_keys = Vec::with_capacity(value.table_len);
            let mut cloned_vals = Vec::with_capacity(value.table_len);

            for i in 0..value.table_len {
                if !keys_slice[i].is_null() {
                    let c_str = unsafe { CStr::from_ptr(keys_slice[i]) };
                    cloned_keys.push(to_cstring(
                        c_str.to_str().map_err(|e| e.to_string())?.to_string(),
                    ));
                } else {
                    cloned_keys.push(std::ptr::null_mut());
                }

                cloned_vals.push(unsafe { clone_ffi_value(&vals_slice[i])? });
            }

            let len = cloned_keys.len();
            let keys_ptr = Box::into_raw(cloned_keys.into_boxed_slice()) as *mut *mut c_char;
            let vals_ptr = Box::into_raw(cloned_vals.into_boxed_slice()) as *mut FFIValue;

            Ok(FFIValue {
                kind: value.kind,
                table_keys: keys_ptr,
                table_vals: vals_ptr,
                table_len: len,
                ..Default::default()
            })
        }
    }
}

// Config Saving and Loading

#[unsafe(no_mangle)]
extern "C" fn config_save_exposed() -> FFIResult {
    crate::config().save().into()
}

#[unsafe(no_mangle)]
extern "C" fn config_load_exposed() -> () {
    crate::config().load();
}

#[unsafe(no_mangle)]
extern "C" fn config_directory_exposed() -> *const c_char {
    match crate::config().config_directory() {
        Some(p) => to_cstring(p.to_string_lossy().to_string()),
        None => std::ptr::null_mut(),
    }
}

// Config querying

#[unsafe(no_mangle)]
extern "C" fn config_get_key_exposed(section: *const c_char, key: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);
    let key = cstr_arg!(key, FFIResult);
    crate::config().get_key(section, key).into()
}

#[unsafe(no_mangle)]
extern "C" fn config_contains_key_exposed(section: *const c_char, key: *const c_char) -> bool {
    let section = cstr_arg!(section, bool);
    let key = cstr_arg!(key, bool);
    crate::config().contains_key(section, key).unwrap_or(false)
}

#[unsafe(no_mangle)]
extern "C" fn config_list_sections_exposed(outlen: *mut c_int) -> *const *mut c_char {
    let sections = crate::config().list_sections();
    let sections_raw: Vec<_> = sections.into_iter().map(|s| to_cstring(s)).collect();

    unsafe { vec_to_ffi_array(sections_raw, outlen) }
}

#[unsafe(no_mangle)]
extern "C" fn config_list_keys_exposed(section: *const c_char, outlen: *mut c_int) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    let keys = match crate::config().list_keys(section) {
        Ok(v) => v,
        Err(e) => return FFIResult::err(e.to_string()),
    };

    let keys_raw: Vec<_> = keys.into_iter().map(|s| to_cstring(s)).collect();
    FFIResult::ok(unsafe { vec_to_ffi_array(keys_raw, outlen) })
}

#[unsafe(no_mangle)]
extern "C" fn config_list_values_exposed(section: *const c_char, outlen: *mut c_int) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    let values = match crate::config().list_values(section) {
        Ok(v) => v,
        Err(e) => return FFIResult::err(e.to_string()),
    };

    let values_raw: Vec<_> = values.into_iter().map(FFIValue::from).collect();
    FFIResult::ok(unsafe { vec_to_ffi_array(values_raw, outlen) })
}

#[unsafe(no_mangle)]
extern "C" fn config_list_items_exposed(section: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    match crate::config().list_items(section) {
        Ok(items) => {
            let mut keys: Vec<*mut c_char> = Vec::with_capacity(items.len());
            let mut vals: Vec<FFIValue> = Vec::with_capacity(items.len());

            for (k, v) in items {
                keys.push(to_cstring(k));
                vals.push(FFIValue::from(v));
            }

            let keys_ptr = keys.as_mut_ptr();
            let vals_ptr = vals.as_mut_ptr();
            let len = keys.len();

            std::mem::forget(keys);
            std::mem::forget(vals);

            FFIResult::ok(FFIValue {
                kind: FFIValueKind::Table,
                table_keys: keys_ptr,
                table_vals: vals_ptr,
                table_len: len,
                ..Default::default()
            })
        }
        Err(e) => FFIResult::err(e.to_string()),
    }
}

#[unsafe(no_mangle)]
extern "C" fn config_set_key_exposed(
    section: *const c_char,
    key: *const c_char,
    value: FFIValue,
) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);
    let key = cstr_arg!(key, FFIResult);

    let value = match value.try_into() {
        Ok(v) => v,
        Err(e) => return FFIResult::err(e),
    };

    crate::config().set_key(section, key, value).into()
}

// Backend Switching

#[repr(C)]
#[derive(Debug)]
pub struct FFIBackendVTable {
    pub load_fn: extern "C" fn() -> *mut c_char,
    pub save_fn: extern "C" fn(*const c_char) -> *mut c_char,
    pub dir_fn: extern "C" fn() -> *mut c_char,
}

#[derive(Debug)]
struct FFIBackend {
    vtable: FFIBackendVTable,
}

impl ConfigBackend for FFIBackend {
    fn load(&self) -> Option<String> {
        let raw = (self.vtable.load_fn)();
        if raw.is_null() {
            return None;
        }

        unsafe {
            let s = CStr::from_ptr(raw).to_string_lossy().to_string();
            drop(CString::from_raw(raw));
            Some(s)
        }
    }

    fn save(&self, content: &str) -> Result<(), String> {
        let c = match CString::new(content) {
            Ok(v) => v,
            Err(e) => return Err(e.to_string()),
        };

        let raw = (self.vtable.save_fn)(c.as_ptr());
        if raw.is_null() {
            return Ok(());
        };

        unsafe {
            let s = CStr::from_ptr(raw).to_string_lossy().to_string();
            drop(CString::from_raw(raw));
            Err(s)
        }
    }

    fn config_directory(&self) -> Option<std::path::PathBuf> {
        let raw = (self.vtable.dir_fn)();
        if raw.is_null() {
            return None;
        }

        unsafe {
            let s = CStr::from_ptr(raw).to_string_lossy().to_string();
            drop(CString::from_raw(raw));
            Some(std::path::PathBuf::from(s))
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn config_set_backend_exposed(vtable: FFIBackendVTable) {
    crate::config().set_backend(Box::new(FFIBackend { vtable }));
}
