use nix::sys::pthread::*;
use std::ptr;

#[cfg(target_env = "musl")]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(tid != ptr::null_mut());
}

#[cfg(not(target_env = "musl"))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(tid != 0);
}
