use std::{ffi::CStr, os::raw::c_char, thread};

use crate::{start_server, DEFAULT_ADDRESS};

#[no_mangle]
pub extern "C" fn start_server_exposed(address: *const c_char) {
    let address_to_bind = if address.is_null() {
        DEFAULT_ADDRESS
    } else {
        unsafe { CStr::from_ptr(address) }
            .to_str()
            .expect("Input is not valid UTF-8")
    };

    thread::spawn(|| {
        start_server(Some(address_to_bind));
    });
}
