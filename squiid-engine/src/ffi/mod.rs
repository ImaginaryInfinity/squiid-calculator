use std::ffi::{c_char, c_int, CStr, CString};

use data_structs::{BucketFFI, EngineSignalSetFFI};

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
) -> EngineSignalSetFFI {
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

/// Get the engine's current stack.
///
/// # Arguments
///
/// * `outlen` - A pointer to an integer to store the length of the output array
#[no_mangle]
extern "C" fn get_stack_exposed(outlen: *mut c_int) -> *mut *mut BucketFFI {
    // Create a vector of CStrings from the stack
    let mut stack_ptr: Vec<*mut BucketFFI> = crate::get_stack()
        .iter()
        .map(|b| Box::into_raw(Box::new(BucketFFI::from(b.clone()))))
        .collect();

    stack_ptr.shrink_to_fit();
    // assert that shrink_to_fit worked
    assert!(stack_ptr.len() == stack_ptr.capacity());

    // write the vec length to the pointer that was passed in
    let len = stack_ptr.len();
    unsafe { std::ptr::write(outlen, len as c_int) };

    // get the pointer to the vec that we are returning
    let vec_ptr = stack_ptr.as_mut_ptr();
    std::mem::forget(stack_ptr);

    vec_ptr
}

/// Get the engine's list of currently supported commands.
///
/// # Arguments
///
/// * `outlen` - A pointer to an integer to store the length of the output array
#[no_mangle]
extern "C" fn get_commands_exposed(outlen: *mut c_int) -> *mut *mut c_char {
    // convert Vec of Strings into vec of raw pointers
    let mut commands: Vec<_> = crate::get_commands()
        .into_iter()
        .map(|s| CString::new(s).unwrap().into_raw())
        .collect();

    // shrink capacity of vec
    commands.shrink_to_fit();
    assert!(commands.len() == commands.capacity());

    let len = commands.len();
    // forget pointer so that rust doesnt drop it
    let vec_ptr = commands.as_mut_ptr();
    std::mem::forget(commands);

    // write length to outlen
    unsafe { std::ptr::write(outlen, len as c_int) };

    vec_ptr
}

/// Get the current previous answer from the engine.
#[no_mangle]
extern "C" fn get_previous_answer_exposed() -> *mut BucketFFI {
    Box::into_raw(Box::new(BucketFFI::from(crate::get_previous_answer())))
}

/// Update the previous answer variable in the engine.
///
/// This should be called after a full algebraic statement in algebraic mode,
/// or after each RPN command if in RPN mode.
#[no_mangle]
extern "C" fn update_previous_answer_exposed() -> EngineSignalSetFFI {
    let result = crate::update_previous_answer();

    EngineSignalSetFFI::from(result)
}
