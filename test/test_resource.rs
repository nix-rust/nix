use nix::sys::resource::{Resource, getrlimit, setrlimit};

#[test]
pub fn test_resource_limits() {
    let mut limit = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert!(limit.0 != limit.1);

    let orig_limit = limit;

    limit.0 = limit.1;
    setrlimit(Resource::RLIMIT_STACK, limit).unwrap();

    let limit2 = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(limit.0, limit2.0);
    assert_eq!(limit.1, limit2.1);

    setrlimit(Resource::RLIMIT_STACK, orig_limit).unwrap();

    let final_limit = getrlimit(Resource::RLIMIT_STACK).unwrap();
    assert_eq!(orig_limit.0, final_limit.0);
    assert_eq!(orig_limit.1, final_limit.1);
}
