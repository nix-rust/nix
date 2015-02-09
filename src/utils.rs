use std::ffi::CString;
use std::old_path::{Path};

pub trait ToCStr {
    fn to_c_str(&self) -> CString;
}

impl ToCStr for Path {
    fn to_c_str(&self) -> CString {
        CString::from_slice(self.as_vec())
    }
}

impl<'a> ToCStr for &'a str {
    fn to_c_str(&self) -> CString {
        CString::from_slice(self.as_bytes())
    }
}


impl ToCStr for String {
    fn to_c_str(&self) -> CString {
        CString::from_slice(self.as_bytes())
    }
}
