use std::ffi::{OsStr, OsString, CString, NulError};
use std::path::{Path, PathBuf};
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub trait ToCString {
    fn to_cstring(&self) -> Result<CString, NulError>;
    fn into_cstring(self) -> Result<CString, NulError> where Self: Sized { unimplemented!() }
}

impl ToCString for [u8] {
    fn to_cstring(&self) -> Result<CString, NulError> {
        CString::new(self)
    }
}

impl ToCString for Vec<u8> {
    fn to_cstring(&self) -> Result<CString, NulError> {
        ToCString::to_cstring(&**self)
    }

    fn into_cstring(self) -> Result<CString, NulError> {
        CString::new(self)
    }
}

impl ToCString for str {
    fn to_cstring(&self) -> Result<CString, NulError> {
        CString::new(self.as_bytes())
    }
}

impl ToCString for String {
    fn to_cstring(&self) -> Result<CString, NulError> {
        ToCString::to_cstring(&**self)
    }

    fn into_cstring(self) -> Result<CString, NulError> {
        CString::new(self.into_bytes())
    }
}

impl ToCString for OsStr {
    fn to_cstring(&self) -> Result<CString, NulError> {
        CString::new(self.as_bytes())
    }
}

impl ToCString for OsString {
    fn to_cstring(&self) -> Result<CString, NulError> {
        ToCString::to_cstring(&**self)
    }

    fn into_cstring(self) -> Result<CString, NulError> {
        CString::new(self.into_vec())
    }
}

impl ToCString for Path {
    fn to_cstring(&self) -> Result<CString, NulError> {
        ToCString::to_cstring(self.as_os_str())
    }
}

impl ToCString for PathBuf {
    fn to_cstring(&self) -> Result<CString, NulError> {
        ToCString::to_cstring(self.as_os_str())
    }

    fn into_cstring(self) -> Result<CString, NulError> {
        ToCString::into_cstring(self.into_os_string())
    }
}

// TODO: allow this in consts/statics
#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        unsafe { ::std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const _) }
    }
}
