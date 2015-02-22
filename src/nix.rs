use libc;
use std;

use errno::Errno;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    Sys(Errno),
    InvalidPath
}

pub trait NixPath {
    fn with_nix_path<T, F>(&self, f: F) -> std::result::Result<T, Error>
        where F: FnOnce(*const libc::c_char) -> T;
}

impl<'a> NixPath for &'a [u8] {
    fn with_nix_path<T, F>(&self, f: F) -> std::result::Result<T, Error>
        where F: FnOnce(*const libc::c_char) -> T
    {
        // TODO: Extract this size as a const
        let mut buf = [0u8; 4096];

        if self.len() >= 4096 {
            return Err(Error::InvalidPath);
        }

        match self.position_elem(&0) {
            Some(_) => Err(Error::InvalidPath),
            None => {
                std::slice::bytes::copy_memory(&mut buf, self);
                Ok(f(buf.as_ptr() as *const libc::c_char))
            }
        }
    }
}

impl<P: NixPath> NixPath for Option<P> {
    fn with_nix_path<T, F>(&self, f: F) -> std::result::Result<T, Error>
        where F: FnOnce(*const libc::c_char) -> T
    {
        match *self {
            Some(ref some) => some.with_nix_path(f),
            None           => b"".with_nix_path(f)
        }
    }
}

#[inline]
pub fn from_ffi(res: libc::c_int) -> Result<()> {
    if res != 0 {
        return Err(Error::Sys(Errno::last()));
    }
    Ok(())
}
