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
    let name = String::from("nix-rust");
    pthread_setname_np(tid, name.clone());
    let ret = pthread_getname_np(tid).unwrap();
    assert!(name == ret);
}
