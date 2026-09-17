use std::ffi::{CStr, CString, c_char, c_int};

use crate::{
    config_handler::ConfigBackend,
    ffi::{
        config_manager::data_structs::{FFIResult, FFIValue, FFIValueKind},
        utils::{string_to_ffi, vec_to_ffi_array},
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
        Some(p) => string_to_ffi(p.to_string_lossy()),
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
extern "C" fn config_list_sections_exposed() -> FFIResult {
    let sections = crate::config().list_sections();
    let sections_raw: Vec<_> = sections.into_iter().map(FFIValue::from).collect();

    let mut len = 0;
    let ptr = unsafe { vec_to_ffi_array(sections_raw, &mut len as *mut c_int) };

    FFIResult::ok(FFIValue {
        kind: FFIValueKind::Array,
        array: ptr,
        array_len: len as usize,
        ..Default::default()
    })
}

#[unsafe(no_mangle)]
extern "C" fn config_list_items_exposed(section: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    match crate::config().list_items(section) {
        Ok(items) => {
            let mut keys: Vec<*mut c_char> = Vec::with_capacity(items.len());
            let mut vals: Vec<FFIValue> = Vec::with_capacity(items.len());

            for (k, v) in items {
                keys.push(string_to_ffi(k));
                vals.push(FFIValue::from(v));
            }

            let len = keys.len();
            let keys_ptr = unsafe { vec_to_ffi_array(keys, std::ptr::null_mut()) };
            let vals_ptr = unsafe { vec_to_ffi_array(vals, std::ptr::null_mut()) };

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
    value: *mut FFIValue,
) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);
    let key = cstr_arg!(key, FFIResult);

    if value.is_null() {
        return FFIResult::err("value cannot be null");
    }

    let mut ffi_value = unsafe { Box::from_raw(value) };
    let value = match (&*ffi_value).try_into() {
        Ok(v) => v,
        Err(e) => {
            unsafe { ffi_value.free() };
            return FFIResult::err(e);
        }
    };

    unsafe { ffi_value.free() };

    crate::config().set_key(section, key, value).into()
}

#[unsafe(no_mangle)]
extern "C" fn config_create_section_exposed(section: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    crate::config().create_section(section).into()
}

#[unsafe(no_mangle)]
extern "C" fn config_delete_section_exposed(section: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);

    crate::config().delete_section(section).into()
}

#[unsafe(no_mangle)]
extern "C" fn config_delete_key_exposed(section: *const c_char, key: *const c_char) -> FFIResult {
    let section = cstr_arg!(section, FFIResult);
    let key = cstr_arg!(key, FFIResult);

    crate::config().delete_key(section, key).into()
}

#[unsafe(no_mangle)]
extern "C" fn config_merge_with_default_exposed(default: *const c_char) -> FFIResult {
    let default = cstr_arg!(default, FFIResult);

    crate::config().merge_with_default(default).into()
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
