use std::mem;
use libc::{c_char};
use std::ffi::{c_str_to_bytes_with_nul};
use std::str::from_utf8_unchecked; 

mod ffi {
    use libc::c_int;
    use super::UtsName;

    extern {
        pub fn uname(buf: *mut UtsName) -> c_int;
    }
}


const UTSNAME_LEN: usize = 65;

#[repr(C)]
#[derive(Copy)]
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
        to_str(&(&self.sysname as *const c_char ) as *const *const c_char)
    }

    pub fn nodename<'a>(&'a self) -> &'a str {
        to_str(&(&self.nodename as *const c_char ) as *const *const c_char)
    }

    pub fn release<'a>(&'a self) -> &'a str {
        to_str(&(&self.release as *const c_char ) as *const *const c_char)
    }

    pub fn version<'a>(&'a self) -> &'a str {
        to_str(&(&self.version as *const c_char ) as *const *const c_char)
    }

    pub fn machine<'a>(&'a self) -> &'a str {
        to_str(&(&self.machine as *const c_char ) as *const *const c_char)
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
fn to_str<'a>(s: *const *const c_char) -> &'a str {
    unsafe {
        let res = c_str_to_bytes_with_nul(mem::transmute(s));
        from_utf8_unchecked(res)
    }
}
