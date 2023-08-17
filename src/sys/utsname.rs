//! Get system identification
use crate::{Errno, Result};
use libc::c_char;
use core::ffi::CStr;
use core::mem;
use core::os::unix::ffi::CStrExt;

/// Describes the running system.  Return type of [`uname`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct UtsName(libc::utsname);

impl UtsName {
    /// Name of the operating system implementation.
    pub fn sysname(&self) -> &CStr {
        cast_and_trim(&self.0.sysname)
    }

    /// Network name of this machine.
    pub fn nodename(&self) -> &CStr {
        cast_and_trim(&self.0.nodename)
    }

    /// Release level of the operating system.
    pub fn release(&self) -> &CStr {
        cast_and_trim(&self.0.release)
    }

    /// Version level of the operating system.
    pub fn version(&self) -> &CStr {
        cast_and_trim(&self.0.version)
    }

    /// Machine hardware platform.
    pub fn machine(&self) -> &CStr {
        cast_and_trim(&self.0.machine)
    }

    /// NIS or YP domain name of this machine.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    pub fn domainname(&self) -> &CStr {
        cast_and_trim(&self.0.domainname)
    }
}

/// Get system identification
pub fn uname() -> Result<UtsName> {
    unsafe {
        let mut ret = mem::MaybeUninit::zeroed();
        Errno::result(libc::uname(ret.as_mut_ptr()))?;
        Ok(UtsName(ret.assume_init()))
    }
}

fn cast_and_trim(slice: &[c_char]) -> &CStr {
    let length = slice
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(slice.len());
    let bytes =
        unsafe { core::slice::from_raw_parts(slice.as_ptr().cast(), length) };

    CStr::from_bytes(bytes)
}

#[cfg(test)]
mod test {
    #[cfg(target_os = "linux")]
    #[test]
    pub fn test_uname_linux() {
        assert_eq!(super::uname().unwrap().sysname(), "Linux");
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    pub fn test_uname_darwin() {
        assert_eq!(super::uname().unwrap().sysname(), "Darwin");
    }

    #[cfg(target_os = "freebsd")]
    #[test]
    pub fn test_uname_freebsd() {
        assert_eq!(super::uname().unwrap().sysname(), "FreeBSD");
    }
}
