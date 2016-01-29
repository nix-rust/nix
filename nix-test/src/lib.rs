extern crate libc;

use std::fmt;
use std::ffi::CString;
use libc::{c_int, c_char};

mod ffi {
    use libc::{c_int, c_char, size_t};

    #[link(name = "nixtest", kind = "static")]
    extern {
        pub fn get_int_const(errno: *const c_char) -> c_int;
        pub fn size_of(ty: *const c_char) -> size_t;
    }
}

pub fn assert_const_eq<T: GetConst>(name: &str, actual: T) {
    unsafe {
        let cstr = CString::new(name).unwrap();
        let expect = GetConst::get_const(cstr.as_ptr());

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

pub use ffi::get_int_const;

pub trait GetConst : PartialEq<Self> + fmt::Display {
    unsafe fn get_const(name: *const c_char) -> Self;
}

impl GetConst for c_int {
    unsafe fn get_const(name: *const c_char) -> c_int {
        ffi::get_int_const(name)
    }
}

impl GetConst for u32 {
    unsafe fn get_const(name: *const c_char) -> u32 {
        ffi::get_int_const(name) as u32
    }
}
