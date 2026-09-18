use std::ffi::{CStr, CString, c_char};

use crate::ffi::utils::{squiid_string_clone, string_to_ffi};

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
#[derive(Default, Copy, Clone)]
pub struct FFIDatetime {
    pub has_date: bool,
    pub year: u16,
    pub month: u8,
    pub day: u8,

    pub has_time: bool,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub has_second: bool,
    pub nanosecond: u32,
    pub has_nanosecond: bool,

    pub has_offset: bool,
    pub is_offset_z: bool,
    /// this is set if [`is_offset_z`] is false
    pub offset_minutes: i16,
}

impl From<toml::value::Datetime> for FFIDatetime {
    fn from(value: toml::value::Datetime) -> Self {
        let date = value.date;
        let time = value.time;
        let offset = value.offset;
        Self {
            has_date: date.is_some(),
            year: date.map(|d| d.year).unwrap_or_default(),
            month: date.map(|d| d.month).unwrap_or_default(),
            day: date.map(|d| d.day).unwrap_or_default(),

            has_time: time.is_some(),
            hour: time.map(|t| t.hour).unwrap_or_default(),
            minute: time.map(|t| t.minute).unwrap_or_default(),
            second: time
                .map(|t| t.second.unwrap_or_default())
                .unwrap_or_default(),
            has_second: time.map(|t| t.second.is_some()).unwrap_or(false),
            nanosecond: time
                .map(|t| t.nanosecond.unwrap_or_default())
                .unwrap_or_default(),
            has_nanosecond: time.map(|t| t.nanosecond.is_some()).unwrap_or(false),

            has_offset: offset.is_some(),
            is_offset_z: offset
                .map(|o| matches!(o, toml::value::Offset::Z))
                .unwrap_or(false),
            offset_minutes: offset
                .map(|o| {
                    if let toml::value::Offset::Custom { minutes } = o {
                        minutes
                    } else {
                        0
                    }
                })
                .unwrap_or_default(),
        }
    }
}

impl From<&FFIDatetime> for toml::value::Datetime {
    fn from(value: &FFIDatetime) -> Self {
        let date = if value.has_date {
            Some(toml::value::Date {
                year: value.year,
                month: value.month,
                day: value.day,
            })
        } else {
            None
        };

        let time = if value.has_time {
            Some(toml::value::Time {
                hour: value.hour,
                minute: value.minute,
                second: if value.has_second {
                    Some(value.second)
                } else {
                    None
                },
                nanosecond: if value.has_nanosecond {
                    Some(value.nanosecond)
                } else {
                    None
                },
            })
        } else {
            None
        };

        let offset = if value.has_offset {
            if value.is_offset_z {
                Some(toml::value::Offset::Z)
            } else {
                Some(toml::value::Offset::Custom {
                    minutes: value.offset_minutes,
                })
            }
        } else {
            None
        };

        toml::value::Datetime { date, time, offset }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct FFIValue {
    pub kind: FFIValueKind,
    pub string_val: *mut c_char,
    pub int_val: i64,
    pub float_val: f64,
    pub bool_val: bool,
    pub datetime_val: *mut FFIDatetime,
    pub array: *mut FFIValue,
    pub array_len: usize,
    pub table_keys: *mut *mut c_char,
    pub table_vals: *mut FFIValue,
    pub table_len: usize,
}

impl From<toml::Value> for FFIValue {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) => s.into(),
            toml::Value::Integer(i) => i.into(),
            toml::Value::Float(f) => f.into(),
            toml::Value::Boolean(b) => b.into(),
            toml::Value::Datetime(datetime) => Self {
                kind: FFIValueKind::Datetime,
                datetime_val: Box::into_raw(Box::new(datetime.into())),
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
                    keys.push(string_to_ffi(k));
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

macro_rules! from_impl_ffivalue {
    ($ty:ty, $kind:path, $member:ident) => {
        impl From<$ty> for FFIValue {
            fn from(value: $ty) -> Self {
                Self {
                    kind: $kind,
                    $member: value,
                    ..Default::default()
                }
            }
        }
    };
    ($ty:ty, $kind:path, $member:ident, $convert_fn:ident) => {
        impl From<$ty> for FFIValue {
            fn from(value: $ty) -> Self {
                Self {
                    kind: $kind,
                    $member: $convert_fn(value),
                    ..Default::default()
                }
            }
        }
    };
}

from_impl_ffivalue!(String, FFIValueKind::String, string_val, string_to_ffi);
from_impl_ffivalue!(&str, FFIValueKind::String, string_val, string_to_ffi);
from_impl_ffivalue!(i64, FFIValueKind::Integer, int_val);
from_impl_ffivalue!(f64, FFIValueKind::Float, float_val);
from_impl_ffivalue!(bool, FFIValueKind::Boolean, bool_val);

impl FFIValue {
    pub unsafe fn free(&mut self) {
        unsafe {
            match self.kind {
                FFIValueKind::String => {
                    if !self.string_val.is_null() {
                        drop(CString::from_raw(self.string_val));
                    }
                }
                FFIValueKind::Datetime => {
                    if !self.datetime_val.is_null() {
                        drop(Box::from_raw(self.datetime_val));
                    }
                }
                FFIValueKind::Array => {
                    if !self.array.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.array, self.array_len);
                        let mut items = Box::from_raw(slice);

                        for item in items.iter_mut() {
                            item.free();
                        }

                        drop(items);
                    }
                }
                FFIValueKind::Table => {
                    if !self.table_keys.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.table_keys, self.table_len);
                        let keys = Box::from_raw(slice);

                        for k in keys.iter() {
                            if !k.is_null() {
                                drop(CString::from_raw(*k));
                            }
                        }

                        drop(keys);
                    }

                    if !self.table_vals.is_null() {
                        let slice = std::slice::from_raw_parts_mut(self.table_vals, self.table_len);
                        let mut vals = Box::from_raw(slice);

                        for item in vals.iter_mut() {
                            item.free();
                        }

                        drop(vals);
                    }
                }
                FFIValueKind::Integer | FFIValueKind::Float | FFIValueKind::Boolean => (),
            }
        }
    }
}

impl TryFrom<&FFIValue> for toml::Value {
    type Error = String;

    fn try_from(value: &FFIValue) -> Result<Self, Self::Error> {
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
                if value.datetime_val.is_null() {
                    return Err("FFIValue::Datetime mustn't be null".to_owned());
                }

                let dt = unsafe { &*value.datetime_val };
                Ok(toml::Value::Datetime(dt.into()))
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
                    result.push(
                        item.try_into()
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

                    let val = (&vals_slice[i])
                        .try_into()
                        .map_err(|e| format!("Table value for key {key}: {e}"))?;
                    result.insert(key, val);
                }

                Ok(toml::Value::Table(result))
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_string(s: *const c_char) -> *mut FFIValue {
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::String,
        string_val: squiid_string_clone(s),
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_integer(val: i64) -> *mut FFIValue {
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Integer,
        int_val: val,
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_float(val: f64) -> *mut FFIValue {
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Float,
        float_val: val,
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_boolean(val: bool) -> *mut FFIValue {
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Boolean,
        bool_val: val,
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_datetime(dt: FFIDatetime) -> *mut FFIValue {
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Datetime,
        datetime_val: Box::into_raw(Box::new(dt)),
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_array(val: *const *mut FFIValue, len: usize) -> *mut FFIValue {
    let mut vec = Vec::with_capacity(len);
    for i in 0..len {
        let ptr = unsafe { *val.add(i) };
        if !ptr.is_null() {
            vec.push(unsafe { *Box::from_raw(ptr) });
        }
    }

    let arr = Box::into_raw(vec.into_boxed_slice()) as *mut FFIValue;
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Array,
        array: arr,
        array_len: len,
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_new_table(
    keys: *const *mut c_char,
    vals: *const *mut FFIValue,
    len: usize,
) -> *mut FFIValue {
    let mut my_keys = Vec::with_capacity(len);
    let mut my_vals = Vec::with_capacity(len);
    for i in 0..len {
        my_keys.push(unsafe { squiid_string_clone(*keys.add(i)) });
        let ptr = unsafe { *vals.add(i) };
        if !ptr.is_null() {
            my_vals.push(unsafe { *Box::from_raw(ptr) });
        }
    }

    let table_keys = Box::into_raw(my_keys.into_boxed_slice()) as *mut *mut c_char;
    let table_vals = Box::into_raw(my_vals.into_boxed_slice()) as *mut FFIValue;
    Box::into_raw(Box::new(FFIValue {
        kind: FFIValueKind::Table,
        table_keys,
        table_vals,
        table_len: len,
        ..Default::default()
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_value_free(ptr: *mut FFIValue) {
    if !ptr.is_null() {
        unsafe { Box::from_raw(ptr).free() }
    }
}
