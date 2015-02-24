use libc;
use libc::{c_char};
use std;
use std::ffi::CString;

use errno::Errno;

pub type NixResult<T> = Result<T, NixError>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NixError {
    Sys(Errno),
    InvalidPath
}

pub trait NixPath {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
        where F: FnOnce(*const libc::c_char) -> T;
}

impl<'a> NixPath for &'a [u8] {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
        where F: FnOnce(*const libc::c_char) -> T
    {
        // TODO: Extract this size as a const
        let mut buf = [0u8; 4096];

        if self.len() >= 4096 {
            return Err(NixError::InvalidPath);
        }

        match self.position_elem(&0) {
            Some(_) => Err(NixError::InvalidPath),
            None => {
                std::slice::bytes::copy_memory(&mut buf, self);
                Ok(f(buf.as_ptr() as *const libc::c_char))
            }
        }
    }
}

impl<P: NixPath> NixPath for Option<P> {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
        where F: FnOnce(*const libc::c_char) -> T
    {
        match *self {
            Some(ref some) => some.with_nix_path(f),
            None           => b"".with_nix_path(f)
        }
    }
}

#[inline]
pub fn from_ffi(res: libc::c_int) -> NixResult<()> {
    if res != 0 {
        return Err(NixError::Sys(Errno::last()));
    }
    Ok(())
}

pub trait AsCString {
    fn as_c_char(&self) -> *const c_char;
}

impl<'a> AsCString for &'a str {
    fn as_c_char(&self) -> *const c_char {
        CString::from_slice(self.as_bytes()).as_ptr()
    }
}

impl AsCString for String {
    fn as_c_char(&self) -> *const c_char {
        CString::from_slice(self.as_slice().as_bytes()).as_ptr()
    }
}
