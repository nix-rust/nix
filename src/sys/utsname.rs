use std::mem;
use libc::{self, c_char};
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

#[repr(C)]
#[derive(Copy)]
pub struct UtsName(libc::utsname);

// workaround for `derive(Clone)` not working for fixed-length arrays
impl Clone for UtsName { fn clone(&self) -> UtsName { *self } }

impl UtsName {
    pub fn sysname<'a>(&'a self) -> &'a str {
        to_str(&(&self.0.sysname as *const c_char ) as *const *const c_char)
    }

    pub fn nodename<'a>(&'a self) -> &'a str {
        to_str(&(&self.0.nodename as *const c_char ) as *const *const c_char)
    }

    pub fn release<'a>(&'a self) -> &'a str {
        to_str(&(&self.0.release as *const c_char ) as *const *const c_char)
    }

    pub fn version<'a>(&'a self) -> &'a str {
        to_str(&(&self.0.version as *const c_char ) as *const *const c_char)
    }

    pub fn machine<'a>(&'a self) -> &'a str {
        to_str(&(&self.0.machine as *const c_char ) as *const *const c_char)
    }
}

pub fn uname() -> UtsName {
    unsafe {
        let mut ret: UtsName = mem::uninitialized();
        libc::uname(&mut ret.0);
        ret
    }
}

#[inline]
fn to_str<'a>(s: *const *const c_char) -> &'a str {
    unsafe {
        let res = CStr::from_ptr(*s).to_bytes();
        from_utf8_unchecked(res)
    }
}

#[cfg(test)]
mod test {
    use super::uname;

    #[test]
    pub fn test_uname() {
        assert_eq!(uname().sysname(), "Linux");
    }
}
