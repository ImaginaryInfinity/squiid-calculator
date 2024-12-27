use std::ffi::CString;

use super::data_structs::MessageActionSetFFI;

/// Free the error string contained within the MessageActionSetFFI struct
///
/// # Arguments
///
/// * `ptr` - Pointer to a MessageActionSetFFI struct which was returned from Rust
#[no_mangle]
extern "C" fn free_message_action_set(ptr: MessageActionSetFFI) {
    unsafe {
        if !ptr.error.is_null() {
            let _ = CString::from_raw(ptr.error);
            // the string will be automatically dropped after this
        }
    }
}
