use nix::sys::pthread::*;

#[cfg(any(target_env = "musl", target_os = "redox"))]
#[test]
fn test_pthread_self() {
    let tid = pthread_self();
    assert!(!tid.is_null());
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
#[cfg(target_env = "gnu")]
fn test_pthread_sigqueue_none() {
    use std::ptr::null_mut;
    pthread_sigqueue(pthread_self(), None, SigVal::Int(0)).expect(
        "Should be able to send signal to my thread, with an integer sigval.",
    );
    pthread_sigqueue(pthread_self(), None, SigVal::Ptr(null_mut())).expect(
        "Should be able to send signal to my thread, with an ptr sigval.",
    );
}
