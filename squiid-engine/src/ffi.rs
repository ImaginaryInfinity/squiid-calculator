use std::{ffi::CStr, os::raw::c_char, thread};

use crate::{start_server, DEFAULT_ADDRESS};

#[no_mangle]
pub extern "C" fn start_server_exposed(address: *const c_char, blocking: bool) {
    let address_to_bind = if address.is_null() {
        DEFAULT_ADDRESS
    } else {
        unsafe { CStr::from_ptr(address) }
            .to_str()
            .expect("Input is not valid UTF-8")
    };

    if blocking {
        start_server(Some(address_to_bind))
    } else {
        thread::spawn(|| {
            std::panic::set_hook(Box::new(|panic_info| {
                eprintln!("panic occurred in thread: {:?}", panic_info);
            }));
            start_server(Some(address_to_bind));
        });
    }
}
