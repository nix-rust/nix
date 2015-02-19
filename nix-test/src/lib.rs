#![feature(libc, std_misc)]

extern crate libc;

use std::ffi::CString;
use libc::{c_int};

mod ffi {
    use libc::{c_int, c_char};

    #[link(name = "nixtest", kind = "static")]
    extern {
        pub fn assert_errno_eq(errno: *const c_char) -> c_int;
    }
}

pub fn assert_errno_eq(err: &str, val: c_int) {
    unsafe {
        let name = CString::from_slice(err.as_bytes());
        let actual = ffi::assert_errno_eq(name.as_ptr());

        assert!(actual > 0);

        if val != actual {
            panic!("incorrect value for errno {}; got={}; expected={}",
                   err, val, actual);
        }
    }
}
