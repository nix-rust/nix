use std::ffi::CString;
use std::path::Path;

pub trait ToCStr {
    fn to_c_str(&self) -> CString;
}

impl ToCStr for Path {
    fn to_c_str(&self) -> CString {
        CString::from_slice(self.as_vec())
    }
}

impl ToCStr for String {
    fn to_c_str(&self) -> CString {
        CString::from_slice(self.as_bytes())
    }
}
