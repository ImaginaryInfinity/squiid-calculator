//! This module defines FFI-compatible representations of core engine structures.
//! These structures allow safe data exchange between Rust and other languages (e.g., C).
//!
//! # Overview
//!
//! The FFI (Foreign Function Interface) bindings provided here expose key structures
//! such as [`EngineSignalSetFFI`], [`BucketFFI`], [`BucketTypesFFI`], and [`ConstantTypesFFI`]
//! in a C-compatible format. These types ensure seamless communication across language boundaries
//! while maintaining safety and correctness.
//!
//! # Structures
//!
//! - [`EngineSignalSetFFI`]: Represents signals indicating actions a frontend should take.
//! - [`BucketFFI`]: A foreign-compatible representation of [`Bucket`] containing a value and type metadata.
//! - [`BucketTypesFFI`]: An enum representing different types of buckets (e.g., float, string, constant).
//! - [`ConstantTypesFFI`]: An enum representing mathematical and physical constants (e.g., Pi, Euler's number).
//!
//! # Safety Considerations
//!
//! - Strings (`*mut c_char`) must be properly allocated and freed to avoid memory leaks.
//! - Enum values must match their Rust counterparts to prevent undefined behavior.
//! - Conversions between Rust and FFI types (`From` implementations) ensure type safety.
//! - These types should only be used in an FFI context where proper memory management is guaranteed.

use std::ffi::{c_char, CString};

use crate::{
    bucket::{Bucket, BucketTypes, ConstantTypes},
    EngineSignalSet,
};

/// Struct containing data about which actions a frontend should take next
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct EngineSignalSetFFI {
    /// Whether or not the frontend should fetch the stack
    pub stack_updated: bool,
    /// Whether or not the frontend should quit
    pub quit: bool,
    /// This is set if an error was encountered, or null if not
    pub error: *mut c_char,
}

impl From<EngineSignalSet> for EngineSignalSetFFI {
    fn from(value: EngineSignalSet) -> Self {
        EngineSignalSetFFI {
            stack_updated: value.stack_updated,
            quit: value.quit,
            error: if let Some(error_str) = value.get_error() {
                match CString::new(error_str) {
                    Ok(s) => s.into_raw(),
                    Err(e) => return EngineSignalSet::new().set_error(&e).into(),
                }
            } else {
                std::ptr::null_mut()
            },
        }
    }
}

/// FFI-Compatible [`Bucket`] struct
#[repr(C)]
#[derive(Debug, Clone)]
pub struct BucketFFI {
    /// Bucket value. Will be null when undefined
    pub value: *mut c_char,
    /// The type of the Bucket
    pub bucket_type: BucketTypesFFI,
    /// The type of the constant if `bucket_type` is `Constant`, else will be `Pi`
    pub constant_type: ConstantTypesFFI,
}

impl From<Bucket> for BucketFFI {
    fn from(value: Bucket) -> Self {
        let value_ptr = if let Some(str_val) = value.value {
            match CString::new(str_val) {
                Ok(s) => s.into_raw(),
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        };

        Self {
            value: value_ptr,
            constant_type: match value.bucket_type {
                BucketTypes::Float | BucketTypes::String | BucketTypes::Undefined => {
                    ConstantTypesFFI::Pi
                }
                BucketTypes::Constant(constant_types) => constant_types.into(),
            },
            bucket_type: value.bucket_type.into(),
        }
    }
}

/// FFI-Compatible [`BucketTypes`] enum
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum BucketTypesFFI {
    /// A floating point number. Also contains integers such as 3.0
    Float = 1,
    /// A string
    String,
    /// A constant
    Constant,
    /// Undefined value, such as tan(pi/2)
    Undefined,
}

impl From<BucketTypes> for BucketTypesFFI {
    fn from(value: BucketTypes) -> Self {
        match value {
            BucketTypes::Float => Self::Float,
            BucketTypes::String => Self::String,
            BucketTypes::Constant(_) => Self::Constant,
            BucketTypes::Undefined => Self::Undefined,
        }
    }
}

/// FFI-Compatible [`ConstantTypes`] enum
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum ConstantTypesFFI {
    /// Pi
    Pi = 1,
    /// Pi/2
    HalfPi,
    /// Pi/3
    ThirdPi,
    /// Pi/4
    QuarterPi,
    /// Pi/6
    SixthPi,
    /// Pi/8
    EighthPi,
    /// 2*pi
    TwoPi,
    /// Euler's number
    E,
    /// Speed of light
    C,
    /// Gravitational constant
    G,
    /// Golden ratio
    Phi,
}

impl From<ConstantTypes> for ConstantTypesFFI {
    fn from(value: ConstantTypes) -> Self {
        match value {
            ConstantTypes::Pi => Self::Pi,
            ConstantTypes::HalfPi => Self::HalfPi,
            ConstantTypes::ThirdPi => Self::ThirdPi,
            ConstantTypes::QuarterPi => Self::QuarterPi,
            ConstantTypes::SixthPi => Self::SixthPi,
            ConstantTypes::EighthPi => Self::EighthPi,
            ConstantTypes::TwoPi => Self::TwoPi,
            ConstantTypes::E => Self::E,
            ConstantTypes::C => Self::C,
            ConstantTypes::G => Self::G,
            ConstantTypes::Phi => Self::Phi,
        }
    }
}
