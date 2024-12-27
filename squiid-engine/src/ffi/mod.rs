use std::ffi::{c_char, c_int, CStr, CString};

use data_structs::MessageActionSetFFI;

use crate::execute_multiple_rpn;

mod cleanup;
mod data_structs;

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
    // construct a new vec to hold the data send from the frontend
    let mut rpn_data_vec = Vec::new();

    // iterate over the submissions
    for i in 0..rpn_data_length {
        unsafe {
            // create new strings from the provided pointers and push them to the vec
            let c_str = CStr::from_ptr(*rpn_data.add(i));
            rpn_data_vec.push(c_str.to_str().unwrap());
        }
    }

    // submit all of the commands to the engine
    let result = execute_multiple_rpn(rpn_data_vec);

    // return a struct telling the frontend what to do next
    result.into()
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
