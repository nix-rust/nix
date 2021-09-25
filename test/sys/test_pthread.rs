use nix::sys::pthread::*;

#[cfg(any(target_env = "musl", target_os = "redox"))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(tid != ::std::ptr::null_mut());
}

#[cfg(not(any(target_env = "musl", target_os = "redox")))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(tid != 0);
}

#[test]
#[cfg(not(target_os = "redox"))]
fn test_pthread_kill_none() {
    pthread_kill(pthread_self(), None)
        .expect("Should be able to send signal to my thread.");
}

#[test]
#[cfg(all(target_os = "linux", not(target_env = "musl")))]
fn test_pthread_name_np() {
    let tid = pthread_self();
    let name = b"nix-rust";
    let mut set = pthread_setname_np(tid, name).unwrap();
    let ret = pthread_getname_np(tid).unwrap();

    assert!(set == 0);
    assert!(name == ret.as_bytes());

    let longname = "threadnametoolongtohold";
    set = pthread_setname_np(tid, longname).unwrap();

    assert!(set == libc::ERANGE);

    let badstr = "truncated\0name";
    set = pthread_setname_np(tid, badstr).unwrap();

    assert!(set == 9);
}
