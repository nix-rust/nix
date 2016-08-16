use nix::sys::event;
use libc;
use std::mem::{size_of, size_of_val};
use std::ptr;

#[test]
/// Check that nix::event::KEvent has the same layout as libc::kevent
/// This would be easier with offsetof operators.
/// https://github.com/rust-lang/rfcs/issues/1144
fn test_struct_kevent() {
    let expected = libc::kevent{ident: 0,
                                filter: 0,
                                flags: 0,
                                fflags: 0,
                                data: 0,
                                udata: ptr::null_mut()};
    let actual = event::KEvent{ ident: 0,
                                filter: event::EventFilter::EVFILT_READ,
                                flags: event::EventFlag::empty(),
                                fflags: event::FilterFlag::empty(),
                                data: 0,
                                udata: 0};
    assert!(size_of::<libc::kevent>() == size_of::<event::KEvent>());
    assert!(size_of_val(&expected.ident) == size_of_val(&actual.ident));
    assert!(size_of_val(&expected.filter) == size_of_val(&actual.filter));
    assert!(size_of_val(&expected.flags) == size_of_val(&actual.flags));
    assert!(size_of_val(&expected.fflags) == size_of_val(&actual.fflags));
    assert!(size_of_val(&expected.data) == size_of_val(&actual.data));
    assert!(size_of_val(&expected.udata) == size_of_val(&actual.udata));
}
