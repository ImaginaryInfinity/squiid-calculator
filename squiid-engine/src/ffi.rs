use std::ffi::{c_char, c_int, CStr, CString};

use crate::execute_multiple_rpn;

#[repr(C)]
pub struct MessageActionSetFFI {
    get_stack: bool,
    get_commands: bool,
    get_prev_answer: bool,
    quit: bool,
    error: *mut c_char,
}

/// FFI-Exposed function to submit multiple RPN commands to the engine.
///
/// # Arguments
///
/// * `rpn_data` - the RPN data array of strings to execute
/// * `rpn_data_length` - the length of `rpn_data`
///
/// # Safety
///
/// This function is unsafe because it is exposed over the FFI boundary. It dereferences a pointer
/// to access the rpn_data array
#[no_mangle]
extern "C" fn execute_multiple_rpn_exposed(
    rpn_data: *const *const c_char,
    rpn_data_length: usize,
) -> MessageActionSetFFI {
    let mut rpn_data_vec = Vec::new();
    for i in 0..rpn_data_length {
        unsafe {
            let c_str = CStr::from_ptr(*rpn_data.add(i));
            rpn_data_vec.push(c_str.to_str().unwrap());
        }
    }

    let result = execute_multiple_rpn(rpn_data_vec);

    MessageActionSetFFI {
        get_stack: result.get_stack,
        get_commands: result.get_commands,
        get_prev_answer: result.get_prev_answer,
        quit: result.quit,
        error: if let Some(error_str) = result.get_error() {
            CString::new(error_str).unwrap().into_raw()
        } else {
            std::ptr::null_mut()
        },
    }
}

#[no_mangle]
extern "C" fn free_message_action_set(ptr: MessageActionSetFFI) {
    unsafe {
        if !ptr.error.is_null() {
            let _ = CString::from_raw(ptr.error);
            println!("really dropped");
            // the string will be automatically dropped after this
        }
    }
}

#[no_mangle]
extern "C" fn get_stack(outlen: *mut c_int) -> *mut *mut c_char {
    // TODO: this is very rough for testing, make this better
    let mut stack: Vec<_> = crate::get_stack()
        .iter()
        .map(|i| CString::new(i.to_string()).unwrap().into_raw())
        .collect();
    stack.shrink_to_fit();

    let len = stack.len();
    let vec_ptr = stack.as_mut_ptr();
    std::mem::forget(stack);

    unsafe { std::ptr::write(outlen, len as c_int) };

    vec_ptr
}
