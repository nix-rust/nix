use nix::sys::resource::{Resource, getrlimit, setrlimit};

/// Tests the RLIMIT_NOFILE functionality of getrlimit(), where the resource RLIMIT_NOFILE refers
/// to the maximum file descriptor number that can be opened by the process (aka the maximum number
/// of file descriptors that the process can open, since Linux 4.5).
///
/// We first fetch the existing file descriptor maximum values using getrlimit(), then edit the
/// soft limit to make sure it has a new and distinct value to the hard limit. We then setrlimit()
/// to put the new soft limit in effect, and then getrlimit() once more to ensure the limits have
/// been updated.
#[test]
pub fn test_resource_limits_nofile() {
    let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();

    // make sure the soft limit and hard limit are not equal
    let soft_limit = match soft_limit {
        Some(nofile) => Some(nofile - 1),
        None => Some(1024),
    };
    setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();

    let (new_soft_limit, _new_hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
    assert_eq!(new_soft_limit, soft_limit);
}

/// Tests the RLIMIT_STACK functionality of getrlimit(), where the resource RLIMIT_STACK refers to
/// the maximum stack size that can be spawned by the current process before SIGSEGV is generated. 
///
/// We first save the current stack limits, then newly set the soft limit to the same size as the
/// hard limit. We check to make sure these limits have been updated properly. We then set the
/// stack limits back to the original values, and make sure they have been updated properly.
#[test]
pub fn test_resource_limits_stack() {
    let (mut soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_STACK).unwrap();
    let orig_limit = (soft_limit, hard_limit);

    soft_limit = hard_limit;
    setrlimit(Resource::RLIMIT_STACK, soft_limit, hard_limit).unwrap();

    let limit2 = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(soft_limit, limit2.0);
    assert_eq!(hard_limit, limit2.1);

    setrlimit(Resource::RLIMIT_STACK, orig_limit.0, orig_limit.1).unwrap();

    let final_limit = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(orig_limit.0, final_limit.0);
    assert_eq!(orig_limit.1, final_limit.1);
}
