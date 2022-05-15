use nix::net::if_::*;

#[cfg(any(target_os = "android", target_os = "linux"))]
const LOOPBACK: &[u8] = b"lo";

#[cfg(not(any(target_os = "android", target_os = "linux", target_os = "haiku")))]
const LOOPBACK: &[u8] = b"lo0";

#[cfg(target_os = "haiku")]
const LOOPBACK: &[u8] = b"loop";

#[test]
fn test_if_nametoindex() {
    assert!(if_nametoindex(LOOPBACK).is_ok());
}
