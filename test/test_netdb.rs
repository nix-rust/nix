use nix::netdb::{getaddrinfo, AddressInfoError};

#[test]
fn test_getaddrinfo_all_null() {
    assert_eq!(
        getaddrinfo(None, None, None),
        Err(AddressInfoError::EAI_NONAME)
    )
}
