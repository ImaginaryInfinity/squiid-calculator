//! # FFI Version Compatibility Checking
//!
//! This module provides functionality for checking the compatibility of a binding's version
//! constraint against the version of the dynamic library. It is designed to be used over
//! FFI (Foreign Function Interface) and ensures that bindings interact with a compatible
//! version of the library.
//!
//! ## Features
//!
//! - **Expose Library Version**: The library version is retrieved from Cargo metadata.
//! - **Version Compatibility Check**: Ensures a binding's declared compatibility matches the
//!   current library version using the [`semver`] crate.
//! - **Error Handling Over FFI**: Errors are returned via pointers to allow bindings to read
//!   meaningful messages in case of mismatches or invalid input.
//!
//! ## Functions
//!
//! - [`check_compatible`]: Checks if a given version constraint is compatible with the library.
//! - [`write_error`]: Writes an error message to a C-compatible string pointer for FFI consumers.
//!
//! ## Usage
//!
//! This module is meant to be used over FFI and follows common C interop conventions.
//!
//! ```c
//! #define COMPATIBLE_ENGINE_VERSION ">=2.1.0,<3.0"
//!
//! char* error_message = NULL;
//! bool compatible = check_compatible(COMPATIBLE_ENGINE_VERSION, &error_message);
//!
//! if (!compatible) {
//!     printf("Incompatible version: %s\n", error_message);
//!     free(error_message);
//! }
//! ```
//!
//! ## Safety
//!
//! - The functions in this module handle `null` pointers gracefully but require that valid
//!   UTF-8 strings be provided for version constraints.
//! - The caller is responsible for freeing any allocated error messages returned through
//!   `expected_version_out`.

use std::ffi::{c_char, CStr, CString};

use semver::{Version, VersionReq};

/// The current version of the dynamic library. Used to expose to bindings over FFI.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check that a binding is compatible with this version of the library.
///
/// # Arguments
///
/// * `version_constraint` - the version constraint that the binding is compatible with. See the
///   `semver` crate for the format.
/// * `expected_version_out` - reference to a string that can be written to for error messages, or NULL to
///   ignore errors
#[unsafe(no_mangle)]
pub extern "C" fn check_compatible(
    version_constraint: *const c_char,
    expected_version_out: *mut *mut c_char,
) -> bool {
    // check that version constraint is set
    if version_constraint.is_null() {
        write_error(expected_version_out, "version constraint is not set");
        return false;
    }

    // convert cstr to rust str
    let c_str = unsafe { CStr::from_ptr(version_constraint) };

    let Ok(constraint_str) = c_str.to_str() else {
        write_error(
            expected_version_out,
            "version constraint is not valid utf-8",
        );
        return false;
    };

    let Ok(lib_version) = Version::parse(VERSION) else {
        write_error(expected_version_out, "crate version is malformed");
        return false;
    };

    let Ok(version_req) = VersionReq::parse(constraint_str) else {
        write_error(expected_version_out, "version constraint is malformed");
        return false;
    };

    let compat = version_req.matches(&lib_version);

    if !compat {
        write_error(
            expected_version_out,
            &format!(
                "the library version {} is not compatible with the binding version constraint {}",
                VERSION, constraint_str
            ),
        );
    }

    compat
}

/// Write a string as a [`CString`] to a pointer, usually in the context of error messages.
///
/// # Arguments
///
/// * `dest` - Where to write the message
/// * `value` - The message to write
///
/// # Panics
///
/// If a [`CString`] could not be created from `value`, or if `dest` is a invalid, non-null pointer.
fn write_error(dest: *mut *mut c_char, value: &str) {
    if !dest.is_null() {
        #[allow(clippy::panic)]
        let raw_err = CString::new(value)
            .unwrap_or_else(|_| panic!("Could not create a CString error message with {}", value))
            .into_raw();

        unsafe { *dest = raw_err };
    }
}
