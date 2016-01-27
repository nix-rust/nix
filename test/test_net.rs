use nix::net::if_::*;

#[cfg(target_os = "linux")]
const LOOPBACK: &'static [u8] = b"lo";

#[cfg(not(target_os = "linux"))]
const LOOPBACK: &'static [u8] = b"lo0";

#[test]
fn test_if_nametoindex() {
    assert!(if_nametoindex(&LOOPBACK[..]).is_ok());
}
