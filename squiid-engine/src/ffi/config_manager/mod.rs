use std::ffi::{c_char, c_int, CStr, CString};

use crate::{
    config_handler::ConfigBackend,
    ffi::config_manager::data_structs::{FFIResult, FFIValue},
};

mod data_structs;

macro_rules! cstr_arg {
    ($ptr:ident, FFIResult) => {{
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

    ($ptr:ident, bool) => {{
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

// TODO: maybe move this up 1 mod.rs
macro_rules! to_cstring {
    ($s:expr) => {{
        match std::ffi::CString::new($s) {
            Ok(c) => c.into_raw(),
            Err(_) => std::ptr::null_mut(),
        }
    }};
}
pub(super) use to_cstring;

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
        Some(p) => to_cstring!(p.to_string_lossy().to_string()),
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
    let sections_len = sections.len();

    let mut sections_raw: Vec<_> = sections
        .into_iter()
        .filter_map(|s| CString::new(s).ok().map(|c| c.into_raw()))
        .collect();

    if sections_raw.len() != sections_len {
        unsafe { std::ptr::write(outlen, 0) };
        return std::ptr::null();
    }

    sections_raw.shrink_to_fit();
    assert!(sections_raw.len() == sections_raw.capacity());

    unsafe { std::ptr::write(outlen, sections_len as c_int) };

    sections_raw.as_ptr()
}

#[unsafe(no_mangle)]
extern "C" fn config_list_keys_exposed(section: *const c_char, outlen: *mut c_int) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    let keys = match crate::config().list_keys(section) {
        Ok(v) => v,
        Err(e) => {
            return FFIResult::err(e.to_string());
        }
    };
    let keys_len = keys.len();

    let mut keys_raw: Vec<_> = keys
        .into_iter()
        .filter_map(|s| CString::new(s).ok().map(|c| c.into_raw()))
        .collect();

    if keys_raw.len() != keys_len {
        unsafe { std::ptr::write(outlen, 0) };
        return FFIResult::err("transforming key array into FFI array failed");
    }

    keys_raw.shrink_to_fit();
    assert!(keys_raw.len() == keys_raw.capacity());

    unsafe { std::ptr::write(outlen, keys_len as c_int) };

    FFIResult::ok(keys_raw.as_ptr())
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
