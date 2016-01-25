//! Network interface name resolution.
//!
//! Uses Linux and/or POSIX functions to resolve interface names like "eth0"
//! or "socan1" into device numbers.

use libc::{c_char, c_uint};
use std::ffi::{CString, NulError};
use std::io;

/// Error that can occur during interface name resolution.
#[derive(Debug)]
pub enum NameToIndexError {
    /// Failed to allocate a C-style string to for the syscall
    NulError,
    IOError(io::Error),
}

impl From<NulError> for NameToIndexError {
    fn from(_: NulError) -> NameToIndexError {
        NameToIndexError::NulError
    }
}

impl From<io::Error> for NameToIndexError {
    fn from(e: io::Error) -> NameToIndexError {
        NameToIndexError::IOError(e)
    }
}

extern {
    fn if_nametoindex(ifname: *const c_char) -> c_uint;
}

/// Resolve an interface into a interface number.
pub fn name_to_index(name: &str) -> Result<c_uint, NameToIndexError> {
    let name = try!(CString::new(name));

    let if_index;
    unsafe {
        if_index = if_nametoindex(name.as_ptr());
    }

    if if_index == 0 {
        return Err(NameToIndexError::from(io::Error::last_os_error()));
    }

    Ok(if_index)
}
