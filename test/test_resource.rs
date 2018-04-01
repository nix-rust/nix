use nix::sys::resource::{Resource, getrlimit, setrlimit};

#[test]
pub fn test_resource_limits_nofile() {
    let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();

    // make sure the soft limit and hard limit are not equal
    let soft_limit = match soft_limit {
        Some(nofile) => Some(nofile -1),
        None => Some(1024),
    };
    setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();

    let (new_soft_limit, new_hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
    assert!(new_soft_limit != new_hard_limit);
}

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
