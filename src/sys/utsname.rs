use std::mem;
use std::c_str::CString;
use libc::{c_char};

mod ffi {
    use libc::c_int;
    use super::UtsName;

    extern {
        pub fn uname(buf: *mut UtsName) -> c_int;
    }
}


const UTSNAME_LEN: uint = 65;

#[repr(C)]
#[deriving(Copy)]
pub struct UtsName {
    sysname: [c_char; UTSNAME_LEN],
    nodename: [c_char; UTSNAME_LEN],
    release: [c_char; UTSNAME_LEN],
    version: [c_char; UTSNAME_LEN],
    machine: [c_char; UTSNAME_LEN],
    // ifdef _GNU_SOURCE
    #[allow(dead_code)]
    domainname: [c_char; UTSNAME_LEN]
}

impl UtsName {
    pub fn sysname<'a>(&'a self) -> &'a str {
        to_str(&self.sysname as *const c_char)
    }

    pub fn nodename<'a>(&'a self) -> &'a str {
        to_str(&self.nodename as *const c_char)
    }

    pub fn release<'a>(&'a self) -> &'a str {
        to_str(&self.release as *const c_char)
    }

    pub fn version<'a>(&'a self) -> &'a str {
        to_str(&self.version as *const c_char)
    }

    pub fn machine<'a>(&'a self) -> &'a str {
        to_str(&self.machine as *const c_char)
    }
}

pub fn uname() -> UtsName {
    unsafe {
        let mut ret: UtsName = mem::uninitialized();
        ffi::uname(&mut ret as *mut UtsName);
        ret
    }
}

#[inline]
fn to_str<'a>(s: *const c_char) -> &'a str {
    unsafe {
        let res = CString::new(s, false);
        mem::transmute(res.as_str().expect("[BUG] uname field not UTF-8"))
    }
}
