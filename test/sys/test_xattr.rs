#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_setxattr_file_exist() {
    use nix::{
        errno::Errno,
        sys::xattr::{setxattr, SetxattrFlag},
    };
    use std::fs::File;

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_setxattr_file_exist");
    File::create(temp_file_path.as_path()).unwrap();

    let res = setxattr(
        temp_file_path.as_path(),
        "user.test_setxattr_file_exist",
        "",
        SetxattrFlag::empty(),
    );

    match res {
        // The underlying file system does not support EA, skip this test.
        Err(Errno::ENOTSUP) => {}
        // If EA is supported, then no error should occur
        _ => assert!(res.is_ok()),
    }
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_setxattr_file_not_exist() {
    use nix::{
        errno::Errno,
        sys::xattr::{setxattr, SetxattrFlag},
    };

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_setxattr_file_not_exist");

    let res = setxattr(
        temp_file_path.as_path(),
        "user.test_setxattr_file_not_exist",
        "",
        SetxattrFlag::empty(),
    );

    assert_eq!(res, Err(Errno::ENOENT));
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_fsetxattr() {
    use nix::{
        errno::Errno,
        sys::xattr::{fsetxattr, SetxattrFlag},
    };
    use std::{fs::File, os::unix::io::AsRawFd};

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_fsetxattr");
    let temp_file = File::create(temp_file_path.as_path()).unwrap();
    let temp_file_fd = temp_file.as_raw_fd();

    let res = fsetxattr(
        temp_file_fd,
        "user.test_fsetxattr",
        "",
        SetxattrFlag::empty(),
    );

    match res {
        // The underlying file system does not support EA, skip this test.
        Err(Errno::ENOTSUP) => {}
        // If EA is supported, then no error should occur
        _ => assert!(res.is_ok()),
    }
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_listxattr() {
    use nix::{errno::Errno, sys::xattr::listxattr};
    use std::fs::File;

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_listxattr");
    File::create(temp_file_path.as_path()).unwrap();

    let res = listxattr(temp_file_path.as_path());

    match res {
        // The underlying file system does not support EA, skip this test.
        Err(Errno::ENOTSUP) => {}
        // If EA is supported, then no error should occur
        _ => assert!(res.is_ok()),
    }
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_flistxattr() {
    use nix::{errno::Errno, sys::xattr::flistxattr};
    use std::{fs::File, os::unix::io::AsRawFd};

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_flistxattr");
    let temp_file = File::create(temp_file_path.as_path()).unwrap();
    let temp_file_fd = temp_file.as_raw_fd();

    let res = flistxattr(temp_file_fd);

    match res {
        // The underlying file system does not support EA, skip this test.
        Err(Errno::ENOTSUP) => {}
        // If EA is supported, then no error should occur
        _ => assert!(res.is_ok()),
    }
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_getxattr() {
    use nix::{
        errno::Errno,
        sys::xattr::{getxattr, setxattr, SetxattrFlag},
    };
    use std::{ffi::OsString, fs::File};

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_getxattr");
    File::create(temp_file_path.as_path()).unwrap();

    let res = setxattr(
        temp_file_path.as_path(),
        "user.test_getxattr",
        "",
        SetxattrFlag::empty(),
    );

    // The underlying file system does not support EA, skip this test.
    if let Err(Errno::ENOTSUP) = res {
        return;
    }

    // If EA is supported, then no error should occur
    assert!(res.is_ok());

    assert_eq!(
        Ok(OsString::new()),
        getxattr(temp_file_path.as_path(), "user.test_getxattr")
    );
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_fgetxattr() {
    use nix::{
        errno::Errno,
        sys::xattr::{fgetxattr, fsetxattr, SetxattrFlag},
    };
    use std::{ffi::OsString, fs::File, os::unix::io::AsRawFd};

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_fgetxattr");
    let temp_file = File::create(temp_file_path).unwrap();
    let temp_file_fd = temp_file.as_raw_fd();

    let res = fsetxattr(
        temp_file_fd,
        "user.test_fgetxattr",
        "",
        SetxattrFlag::empty(),
    );

    // The underlying file system does not support EA, skip this test.
    if let Err(Errno::ENOTSUP) = res {
        return;
    }

    // If EA is supported, then no error should occur
    assert!(res.is_ok());

    assert_eq!(
        Ok(OsString::new()),
        fgetxattr(temp_file_fd, "user.test_fgetxattr")
    );
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_removexattr_ea_exist() {
    use nix::{
        errno::Errno,
        sys::xattr::{removexattr, setxattr, SetxattrFlag},
    };
    use std::fs::File;

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_removexattr_ea_exist");
    File::create(temp_file_path.as_path()).unwrap();

    let res = setxattr(
        temp_file_path.as_path(),
        "user.test_removexattr_ea_exist",
        "",
        SetxattrFlag::empty(),
    );

    // The underlying file system does not support EA, skip this test.
    if let Err(Errno::ENOTSUP) = res {
        return;
    }

    // If EA is supported, then no error should occur
    assert!(res.is_ok());

    assert!(removexattr(
        temp_file_path.as_path(),
        "user.test_removexattr_ea_exist",
    )
    .is_ok());
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_removexattr_ea_not_exist() {
    use nix::{
        errno::Errno,
        sys::xattr::{removexattr, setxattr, SetxattrFlag},
    };
    use std::fs::File;

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_removexattr_ea_not_exist");
    File::create(temp_file_path.as_path()).unwrap();

    // Here, we use `setxattr(path, "user.*", value, flags)` instead of `listxattr`
    // to test if EA is supported.
    //
    // This is necessary as we need to know whether `user` namespace EA is supported
    // rather than other categories of EA.
    //
    // For example, on `tmpfs`, `trusted` and `security` namespace EAs are
    // supported, but `user` is not.
    if let Err(Errno::ENOTSUP) = setxattr(
        temp_file_path.as_path(),
        "user.ea",
        "",
        SetxattrFlag::empty(),
    ) {
        // The underlying file system does not support EA, skip this test.
        return;
    }

    assert_eq!(
        Err(Errno::ENODATA),
        removexattr(
            temp_file_path.as_path(),
            "user.test_removexattr_ea_not_exist",
        )
    );
}

#[test]
#[cfg(any(target_os = "linux", target_os = "android"))]
fn test_fremovexattr() {
    use nix::{
        errno::Errno,
        sys::xattr::{fremovexattr, fsetxattr, SetxattrFlag},
    };
    use std::{fs::File, os::unix::io::AsRawFd};

    let temp_dir = tempfile::tempdir_in("./").unwrap();
    let temp_file_path = temp_dir.path().join("test_fremovexattr");
    let temp_file = File::create(temp_file_path.as_path()).unwrap();
    let temp_file_fd = temp_file.as_raw_fd();

    let res = fsetxattr(
        temp_file_fd,
        "user.test_fremovexattr",
        "",
        SetxattrFlag::empty(),
    );

    // The underlying file system does not support EA, skip this test.
    if let Err(Errno::ENOTSUP) = res {
        return;
    }

    // If EA is supported, then no error should occur
    assert!(res.is_ok());

    assert!(fremovexattr(temp_file_fd, "user.test_fremovexattr").is_ok());
}
