use libc;
use std::ffi::{OsStr, AsOsStr};
use std::os::unix::OsStrExt;
use std::path::{Path, PathBuf};
use std::slice::bytes;

use errno::{Errno, EINVAL};

pub type NixResult<T> = Result<T, NixError>;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NixError {
    Sys(Errno),
    InvalidPath
}

impl NixError {
    pub fn last() -> NixError {
        NixError::Sys(Errno::last())
    }

    pub fn invalid_argument() -> NixError {
        NixError::Sys(EINVAL)
    }
}

pub trait NixPath {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
        where F: FnOnce(&OsStr) -> T;
}

impl NixPath for [u8] {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
            where F: FnOnce(&OsStr) -> T {
        // TODO: Extract this size as a const
        let mut buf = [0u8; 4096];

        if self.len() >= 4096 {
            return Err(NixError::InvalidPath);
        }

        match self.position_elem(&0) {
            Some(_) => Err(NixError::InvalidPath),
            None => {
                bytes::copy_memory(&mut buf, self);
                Ok(f(<OsStr as OsStrExt>::from_bytes(&buf[..self.len()])))
            }
        }
    }
}

impl NixPath for Path {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
            where F: FnOnce(&OsStr) -> T {
        Ok(f(self.as_os_str()))
    }
}

impl NixPath for PathBuf {
    fn with_nix_path<T, F>(&self, f: F) -> Result<T, NixError>
            where F: FnOnce(&OsStr) -> T {
        Ok(f(self.as_os_str()))
    }
}

#[inline]
pub fn from_ffi(res: libc::c_int) -> NixResult<()> {
    if res != 0 {
        return Err(NixError::Sys(Errno::last()));
    }

    Ok(())
}
