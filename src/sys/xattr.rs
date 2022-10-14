//! Extended Attributes related APIs

#[cfg(any(target_os = "linux", target_os = "android"))]
use crate::{errno::Errno, NixPath, Result};
#[cfg(any(target_os = "linux", target_os = "android"))]
use std::{
    ffi::{CString, OsStr, OsString},
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        io::RawFd,
    },
    ptr::null_mut,
};

libc_bitflags!(
    /// `flags` used in setting EAs
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub struct SetxattrFlag: libc::c_int {
        /// Perform a pure create, which fails if the named attribute exists already.
        XATTR_CREATE;
        /// Perform a pure replace operation, which fails if the named attribute does not already exist.
        XATTR_REPLACE;
    }
);

/// Retrieves the list of extended attribute names associated with the given `path`
/// in the filesystem. If `path` is a symbolic link, it will be dereferenced.
///
/// For more infomation, see [listxattr(2)](https://man7.org/linux/man-pages/man2/listxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn listxattr<P: ?Sized + NixPath>(path: &P) -> Result<Vec<OsString>> {
    // query the buffer size
    let buffer_size = path.with_nix_path(|path| unsafe {
        libc::listxattr(path.as_ptr(), null_mut(), 0)
    })?;

    // no entries, return early
    if buffer_size == 0 {
        return Ok(Vec::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);
    let res = path.with_nix_path(|path| unsafe {
        libc::listxattr(
            path.as_ptr(),
            buffer.as_ptr() as *mut libc::c_char,
            buffer.capacity(),
        )
    })?;

    Errno::result(res).map(|len| {
        unsafe { buffer.set_len(len as usize) };
        buffer[..(len - 1) as usize]
            .split(|&item| item == 0)
            .map(OsStr::from_bytes)
            .map(|str| str.to_owned())
            .collect::<Vec<OsString>>()
    })
}

/// Retrieves the list of extended attribute names associated with the given `path`
/// in the filesystem. If `path` is a symbolic link, the list of names associated
/// with the link *itself* will be returned.
///
/// For more infomation, see [llistxattr(2)](https://man7.org/linux/man-pages/man2/listxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn llistxattr<P: ?Sized + NixPath>(path: &P) -> Result<Vec<OsString>> {
    // query the buffer size
    let buffer_size = path.with_nix_path(|path| unsafe {
        libc::llistxattr(path.as_ptr(), null_mut(), 0)
    })?;

    // no entries, return early
    if buffer_size == 0 {
        return Ok(Vec::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);
    let res = path.with_nix_path(|path| unsafe {
        libc::llistxattr(
            path.as_ptr(),
            buffer.as_ptr() as *mut libc::c_char,
            buffer.capacity(),
        )
    })?;

    Errno::result(res).map(|len| {
        unsafe { buffer.set_len(len as usize) };
        buffer[..(len - 1) as usize]
            .split(|&item| item == 0)
            .map(OsStr::from_bytes)
            .map(|str| str.to_owned())
            .collect::<Vec<OsString>>()
    })
}

/// Retrieves the list of extended attribute names associated with the file
/// specified by the open file descriptor `fd` in the filesystem.
///
/// For more infomation, see [flistxattr(2)](https://man7.org/linux/man-pages/man2/listxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn flistxattr(fd: RawFd) -> Result<Vec<OsString>> {
    // query the buffer size
    let buffer_size = unsafe { libc::flistxattr(fd, null_mut(), 0) };

    // no entries, return early
    if buffer_size == 0 {
        return Ok(Vec::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);
    let res = unsafe {
        libc::flistxattr(
            fd,
            buffer.as_ptr() as *mut libc::c_char,
            buffer.capacity(),
        )
    };

    Errno::result(res).map(|len| {
        unsafe { buffer.set_len(len as usize) };
        buffer[..(len - 1) as usize]
            .split(|&item| item == 0)
            .map(OsStr::from_bytes)
            .map(|str| str.to_owned())
            .collect::<Vec<OsString>>()
    })
}

/// Retrieves the value of the extended attribute identified by `name` and
/// associated with the given `path` in the filesystem. If `path` is a symbolic
/// link, it will be dereferenced.
///
/// For more information, see [getxattr(2)](https://man7.org/linux/man-pages/man2/getxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn getxattr<P, S>(path: &P, name: S) -> Result<OsString>
where
    P: ?Sized + NixPath,
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    // query the buffer size
    let buffer_size = path.with_nix_path(|path| unsafe {
        libc::getxattr(
            path.as_ptr(),
            name.as_ptr() as *mut libc::c_char,
            null_mut(),
            0,
        )
    })?;

    // The corresponding value is empty, return
    if buffer_size == 0 {
        return Ok(OsString::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);

    let res = path.with_nix_path(|path| unsafe {
        libc::getxattr(
            path.as_ptr() as *mut libc::c_char,
            name.as_ptr() as *mut libc::c_char,
            buffer.as_ptr() as *mut libc::c_void,
            buffer_size as usize,
        )
    })?;

    Errno::result(res).map(|len| unsafe {
        buffer.set_len(len as usize);
        OsString::from_vec(buffer)
    })
}

/// Retrieves the value of the extended attribute identified by `name` and
/// associated with the given `path` in the filesystem. If `path` is a symbolic
/// link, the list of names associated with the link *itself* will be returned.
///
/// For more information, see [lgetxattr(2)](https://man7.org/linux/man-pages/man2/getxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lgetxattr<P, S>(path: &P, name: S) -> Result<OsString>
where
    P: ?Sized + NixPath,
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    // query the buffer size
    let buffer_size = path.with_nix_path(|path| unsafe {
        libc::lgetxattr(
            path.as_ptr(),
            name.as_ptr() as *mut libc::c_char,
            null_mut(),
            0,
        )
    })?;

    // The corresponding value is empty, return
    if buffer_size == 0 {
        return Ok(OsString::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);

    let res = path.with_nix_path(|path| unsafe {
        libc::lgetxattr(
            path.as_ptr() as *mut libc::c_char,
            name.as_ptr() as *mut libc::c_char,
            buffer.as_ptr() as *mut libc::c_void,
            buffer_size as usize,
        )
    })?;

    Errno::result(res).map(|len| unsafe {
        buffer.set_len(len as usize);
        OsString::from_vec(buffer)
    })
}

/// Retrieves the value of the extended attribute identified by `name` and
/// associated with the file specified by the open file descriptor `fd` in the
/// filesystem.
///
/// For more information, see [fgetxattr(2)](https://man7.org/linux/man-pages/man2/getxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn fgetxattr<S>(fd: RawFd, name: S) -> Result<OsString>
where
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    // query the buffer size
    let buffer_size = unsafe {
        libc::fgetxattr(fd, name.as_ptr() as *mut libc::c_char, null_mut(), 0)
    };

    // The corresponding value is empty, return
    if buffer_size == 0 {
        return Ok(OsString::new());
    }

    let mut buffer: Vec<u8> =
        Vec::with_capacity(Errno::result(buffer_size)? as usize);

    let res = unsafe {
        libc::fgetxattr(
            fd,
            name.as_ptr() as *mut libc::c_char,
            buffer.as_ptr() as *mut libc::c_void,
            buffer_size as usize,
        )
    };

    Errno::result(res).map(|len| unsafe {
        buffer.set_len(len as usize);
        OsString::from_vec(buffer)
    })
}

/// Removes the extended attribute identified by `name` and associated with the
/// given `path` in the filesystem. If `path` is a symbolic link, it will be
/// dereferenced.
///
/// For more information, see [removexattr(2)](https://man7.org/linux/man-pages/man2/removexattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn removexattr<P, S>(path: &P, name: S) -> Result<()>
where
    P: ?Sized + NixPath,
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };
    let res = path.with_nix_path(|path| unsafe {
        libc::removexattr(path.as_ptr() as *mut libc::c_char, name.as_ptr())
    })?;

    Errno::result(res).map(drop)
}

/// Removes the extended attribute identified by `name` and associated with the
/// given `path` in the filesystem. If `path` is a symbolic link, extended
/// attribute is removed from the link *itself*.
///
/// For more information, see [lremovexattr(2)](https://man7.org/linux/man-pages/man2/removexattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lremovexattr<P, S>(path: &P, name: S) -> Result<()>
where
    P: ?Sized + NixPath,
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };
    let res = path.with_nix_path(|path| unsafe {
        libc::lremovexattr(path.as_ptr() as *mut libc::c_char, name.as_ptr())
    })?;

    Errno::result(res).map(drop)
}

/// Removes the extended attribute identified by `name` and associated with the
/// file specified by the open file descriptor `fd`.
///
/// For more information, see [fremovexattr(2)](https://man7.org/linux/man-pages/man2/removexattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn fremovexattr<S>(fd: RawFd, name: S) -> Result<()>
where
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    let res = unsafe { libc::fremovexattr(fd, name.as_ptr()) };

    Errno::result(res).map(drop)
}

/// Sets the `value` of the extended attribute identified by `name` and associated
/// with the given `path` in the filesystem. If `path` is a symbolic link, it will
/// be dereferenced.
///
/// For more information, see [setxattr(2)](https://man7.org/linux/man-pages/man2/lsetxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn setxattr<P, S>(
    path: &P,
    name: S,
    value: S,
    flags: SetxattrFlag,
) -> Result<()>
where
    P: ?Sized + NixPath,
    S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    let value_ptr = value.as_ref().as_bytes().as_ptr() as *mut libc::c_void;
    let value_len = value.as_ref().len();

    let res = path.with_nix_path(|path| unsafe {
        libc::setxattr(
            path.as_ptr() as *mut libc::c_char,
            name.as_ptr(),
            value_ptr,
            value_len,
            flags.bits,
        )
    })?;

    Errno::result(res).map(drop)
}

/// Sets the `value` of the extended attribute identified by `name` and associated
/// with the given `path` in the filesystem. If `path` is a symbolic link, the
/// extended attribute is set on the link *itself*.
///
/// For more information, see [lsetxattr(2)](https://man7.org/linux/man-pages/man2/lsetxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn lsetxattr<P, S>(
    path: &P,
    name: S,
    value: S,
    flags: SetxattrFlag,
) -> Result<()>
    where
        P: ?Sized + NixPath,
        S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    let value_ptr = value.as_ref().as_bytes().as_ptr() as *mut libc::c_void;
    let value_len = value.as_ref().len();

    let res = path.with_nix_path(|path| unsafe {
        libc::lsetxattr(
            path.as_ptr() as *mut libc::c_char,
            name.as_ptr(),
            value_ptr,
            value_len,
            flags.bits,
        )
    })?;

    Errno::result(res).map(drop)
}

/// Sets the `value` of the extended attribute identified by `name` and associated
/// with the file specified by the open file descriptor `fd`.
///
/// For more information, see [fsetxattr(2)](https://man7.org/linux/man-pages/man2/lsetxattr.2.html)
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn fsetxattr<S>(
    fd: RawFd,
    name: S,
    value: S,
    flags: SetxattrFlag,
) -> Result<()>
    where
        S: AsRef<OsStr>,
{
    let name = if let Ok(name) = CString::new(name.as_ref().as_bytes()) {
        name
    } else {
        // if `name` contains 0 bytes, return EINVAL
        return Err(Errno::EINVAL);
    };

    let value_ptr = value.as_ref().as_bytes().as_ptr() as *mut libc::c_void;
    let value_len = value.as_ref().len();

    let res = unsafe {
        libc::fsetxattr(
            fd,
            name.as_ptr(),
            value_ptr,
            value_len,
            flags.bits,
        )
    };

    Errno::result(res).map(drop)
}
