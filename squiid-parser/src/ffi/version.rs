use std::ffi::{c_char, CStr, CString};

use semver::{Version, VersionReq};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check that a binding is compatible with this version of the library.
///
/// # Arguments
///
/// * `version_constraint` - the version constraint that the binding is compatible with. See the
/// `semver` crate for the format.
/// * `expected_version_out` - reference to a string that can be written to for error messages, or NULL to
/// ignore errors
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
    let constraint_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            write_error(
                expected_version_out,
                "version constraint is not valid utf-8",
            );

            return false;
        }
    };

    let lib_version = match Version::parse(VERSION) {
        Ok(v) => v,
        Err(_) => {
            write_error(expected_version_out, "crate version is malformed");
            return false;
        }
    };

    let version_req = match VersionReq::parse(constraint_str) {
        Ok(req) => req,
        Err(_) => {
            write_error(expected_version_out, "version constraint is malformed");
            return false;
        }
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

fn write_error(dest: *mut *mut c_char, value: &str) {
    if !dest.is_null() {
        unsafe { *dest = CString::new(value).unwrap().into_raw() };
    }
}
