use std::ffi::{c_char, CString};

use crate::MessageActionSet;

/// Struct containing data about which actions a frontend should take next
#[repr(C)]
pub struct MessageActionSetFFI {
    /// Whether or not the frontend should fetch the stack
    pub get_stack: bool,
    /// Whether or not the frontend should fetch the previous answer
    pub get_prev_answer: bool,
    /// Whether or not the frontend should quit
    pub quit: bool,
    /// This is set if an error was encountered, or null if not
    pub error: *mut c_char,
}

impl From<MessageActionSet> for MessageActionSetFFI {
    fn from(value: MessageActionSet) -> Self {
        MessageActionSetFFI {
            get_stack: value.get_stack,
            get_prev_answer: value.get_prev_answer,
            quit: value.quit,
            error: if let Some(error_str) = value.get_error() {
                CString::new(error_str).unwrap().into_raw()
            } else {
                std::ptr::null_mut()
            },
        }
    }
}
