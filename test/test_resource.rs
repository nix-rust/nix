use nix::sys::resource::{Resource, getrlimit, setrlimit};

#[test]
pub fn test_resource_limits() {
    let mut limit = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert!(limit.rlim_cur != limit.rlim_max);

    let orig_limit = limit;

    limit.rlim_cur = limit.rlim_max;
    setrlimit(Resource::RLIMIT_STACK, limit).unwrap();

    let limit2 = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(limit.rlim_cur, limit2.rlim_cur);
    assert_eq!(limit.rlim_max, limit2.rlim_max);

    setrlimit(Resource::RLIMIT_STACK, orig_limit).unwrap();

    let final_limit = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(orig_limit.rlim_cur, final_limit.rlim_cur);
    assert_eq!(orig_limit.rlim_max, final_limit.rlim_max);
}
