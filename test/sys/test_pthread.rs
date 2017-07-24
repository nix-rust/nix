use nix::sys::pthread::*;

#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(tid != 0);
}
