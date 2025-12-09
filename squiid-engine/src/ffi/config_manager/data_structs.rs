use std::ffi::{c_char, c_void, CStr, CString};

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

trait Freeable {
    unsafe fn free_ffi(self);
}

impl Freeable for *mut c_char {
    unsafe fn free_ffi(self) {
        if !self.is_null() {
            drop(unsafe { CString::from_raw(self) });
        }
    }
}

impl Freeable for *mut FFIValue {
    unsafe fn free_ffi(self) {
        if !self.is_null() {
            let mut val = unsafe { Box::from_raw(self) };
            unsafe { val.free() };
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
            extern "C" fn [<free_ $name _result>](ffi_result: *mut FFIResult) {
                if !ffi_result.is_null() {
                    let result = unsafe { Box::from_raw(ffi_result) };
                    if !result.value.is_null() {
                        unsafe { <$ty as Freeable>::free_ffi(result.value as $ty) };
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

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub enum FFIValueKind {
    #[default]
    String = 1,
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

impl FFIValue {
    pub unsafe fn free(&mut self) {
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
