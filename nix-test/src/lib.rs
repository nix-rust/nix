extern crate libc;

use std::ffi::CString;
use libc::{c_int};

mod ffi {
    use libc::{c_int, c_char, size_t};

    #[link(name = "nixtest", kind = "static")]
    extern {
        pub fn assert_errno_eq(errno: *const c_char) -> c_int;
        pub fn size_of(ty: *const c_char) -> size_t;
    }
}

pub fn assert_errno_eq(name: &str, actual: c_int) {
    unsafe {
        let cstr = CString::new(name).unwrap();
        let expect = ffi::assert_errno_eq(cstr.as_ptr());

        assert!(expect > 0, "undefined errno {}", name);

        if actual != expect {
            panic!("incorrect value for errno {}; expect={}; actual={}",
                   name, expect, actual);
        }
    }
}

pub fn assert_size_of<T>(name: &str) {
    use std::mem;

    unsafe {
        let cstr = CString::new(name).unwrap();
        let expect = ffi::size_of(cstr.as_ptr()) as usize;

        assert!(expect > 0, "undefined type {}", name);

        if mem::size_of::<T>() != expect {
            panic!("incorrectly sized type; expect={}; actual={}",
                   expect, mem::size_of::<T>());
        }
    }
}
