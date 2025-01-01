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
                CString::new(error_str).unwrap().into_raw()
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
    /// The type of the constant if bucket_type is Constant, else will be Pi
    pub constant_type: ConstantTypesFFI,
}

impl From<Bucket> for BucketFFI {
    fn from(value: Bucket) -> Self {
        let value_ptr = if let Some(str_val) = value.value {
            let str = CString::new(str_val).unwrap();
            str.into_raw()
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
    Float = 1,
    String,
    Constant,
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
    Pi = 1,
    HalfPi,
    ThirdPi,
    QuarterPi,
    SixthPi,
    EighthPi,
    TwoPi,
    E,
    C,
    G,
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
