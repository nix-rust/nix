#[cfg(target_os = "linux")]
#[test]
#[cfg_attr(miri, ignore)]
pub fn test_uname_linux() {
    assert_eq!(nix::sys::utsname::uname().unwrap().sysname(), "Linux");
}

#[cfg(apple_targets)]
#[test]
#[cfg_attr(miri, ignore)]
pub fn test_uname_darwin() {
    assert_eq!(nix::sys::utsname::uname().unwrap().sysname(), "Darwin");
}

#[cfg(target_os = "freebsd")]
#[cfg_attr(miri, ignore)]
#[test]
pub fn test_uname_freebsd() {
    assert_eq!(nix::sys::utsname::uname().unwrap().sysname(), "FreeBSD");
}
