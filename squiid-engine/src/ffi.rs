use std::{ffi::CStr, os::raw::c_char, path::PathBuf, str::FromStr, thread};

use crate::{start_server, DEFAULT_ADDRESS};

/// Start the server with another programming language over FFI
///
/// # Arguments
///
/// * `address` - The address of the server to start
/// * `blocking` - Whether or not the server should be blocking or async
/// * `crash_report_directory` - The directory to store the crash report, or NULL if not
///
/// # Panics
///
/// If an invalid pointer address is provided for address or crash_report_directory, or if the
/// crash_report_directory is not a directory
///
/// # Safety
///
/// I can't do error handling over an FFI boundary, and this is mainly meant for internal binding
/// creation, so we'll just have to write unit tests.
#[no_mangle]
pub unsafe extern "C" fn start_server_exposed(
    address: *const c_char,
    blocking: bool,
    crash_report_directory: *const c_char,
) {
    let address_to_bind = if address.is_null() {
        DEFAULT_ADDRESS
    } else {
        unsafe { CStr::from_ptr(address) }
            .to_str()
            .expect("Input is not valid UTF-8")
    };

    let report_directory = if crash_report_directory.is_null() {
        None
    } else {
        match PathBuf::from_str(
            unsafe { CStr::from_ptr(crash_report_directory) }
                .to_str()
                .expect("Input is not valid UTF-8"),
        ) {
            Ok(val) => {
                if !val.is_dir() {
                    panic!("{:?} is not a valid directory", val.to_str());
                } else {
                    Some(val)
                }
            }
            Err(e) => panic!("{}", e),
        }
    };

    if blocking {
        start_server(Some(address_to_bind), report_directory)
    } else {
        thread::spawn(|| {
            std::panic::set_hook(Box::new(|panic_info| {
                eprintln!("panic occurred in thread: {:?}", panic_info);
            }));
            start_server(Some(address_to_bind), report_directory);
        });
    }
}
