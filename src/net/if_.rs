//! Network interface name resolution.
//!
//! Uses Linux and/or POSIX functions to resolve interface names like "eth0"
//! or "socan1" into device numbers.

use libc;
use libc::c_uint;
use std::ffi::{CString, NulError};
use ::{Result, Error};

/// Resolve an interface into a interface number.
pub fn if_nametoindex(name: &str) -> Result<c_uint> {
    let name = match CString::new(name) {
        Err(e) => match e { NulError(..) => {
            // A NulError indicates that a '\0' was found inside the string,
            // making it impossible to create valid C-String. To avoid having
            // to create a new error type for this rather rare case,
            // nix::Error's invalid_argument() constructor is used.
            //
            // We match the NulError individually here to ensure to avoid
            // accidentally returning invalid_argument() for errors other than
            // NulError (which currently don't exist).
            return Err(Error::invalid_argument());
        }},
        Ok(s) => s
    };

    let if_index;
    unsafe {
        if_index = libc::if_nametoindex(name.as_ptr());
    }

    if if_index == 0 { Err(Error::last()) } else { Ok(if_index) }
}
