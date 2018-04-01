use nix::sys::resource::{Resource, getrlimit, setrlimit};

#[test]
pub fn test_resource_limits() {
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
