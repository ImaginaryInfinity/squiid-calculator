use std::ffi::c_char;

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
