// The conversion is not useless on all platforms.
#[allow(clippy::useless_conversion)]
#[cfg(target_os = "freebsd")]
#[test]
fn test_chflags() {
    use nix::{
        sys::stat::{fstat, FileFlag},
        unistd::chflags,
    };
    use std::os::unix::io::AsRawFd;
    use tempfile::NamedTempFile;

    let f = NamedTempFile::new().unwrap();

    let initial = fstat(f.as_raw_fd()).unwrap().flags();
    // UF_OFFLINE is preserved by all FreeBSD file systems, but not interpreted
    // in any way, so it's handy for testing.
    let commanded = initial ^ FileFlag::UF_OFFLINE;

    chflags(f.path(), commanded).unwrap();

    let changed = fstat(f.as_raw_fd()).unwrap().flags();

    assert_eq!(commanded, changed);
}
