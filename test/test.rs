#![feature(core, libc, net, path, std_misc)]

extern crate nix;
extern crate libc;
extern crate rand;

mod sys;
mod test_stat;
mod test_unistd;

use nix::NixPath;

#[test]
fn test_nix_path() {
    fn cstr_to_bytes(cstr: &*const libc::c_char, len: usize) -> &[u8] {
        unsafe {
            let cstr = cstr as *const _ as *const *const u8;
            std::slice::from_raw_parts(*cstr, len)
        }
    }

    let bytes = b"abcd";
    let ok = bytes.with_nix_path(|cstr| {
        assert_eq!(b"abcd\0", cstr_to_bytes(&cstr, 5));
    });
    assert!(ok.is_ok());

    let bytes = b"ab\0cd";
    let err = bytes.with_nix_path(|_| {});
    assert!(err.is_err());
}
