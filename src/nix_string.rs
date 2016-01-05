use std::ffi::{CStr, CString};

/// Represents a type that can be converted to a `&CStr` without fail.
///
/// Note: this trait exists in place of `AsRef<CStr>` because is not
/// implemented until Rust 1.7.0
pub trait NixString {
    fn as_ref(&self) -> &CStr;
}

impl NixString for CStr {
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl NixString for CString {
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl<'a, T: ?Sized + NixString> NixString for &'a T {
    fn as_ref(&self) -> &CStr {
        NixString::as_ref(*self)
    }
}

impl<'a, T: ?Sized + NixString> NixString for &'a mut T {
    fn as_ref(&self) -> &CStr {
        NixString::as_ref(*self)
    }
}
