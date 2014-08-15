pub mod stat {
    pub use libc::dev_t;

    use std::fmt;
    use std::io::FilePermission;
    use std::path::Path;
    use libc::mode_t;
    use errno::{SysResult, from_ffi};

    mod ffi {
        use libc::{c_char, c_int, mode_t, dev_t};

        extern {
            pub fn mknod(pathname: *const c_char, mode: mode_t, dev: dev_t) -> c_int;
            pub fn umask(mask: mode_t) -> mode_t;
        }
    }

    bitflags!(
        flags SFlag: mode_t {
            static S_IFREG  = 0o100000,
            static S_IFCHR  = 0o020000,
            static S_IFBLK  = 0o060000,
            static S_IFIFO  = 0o010000,
            static S_IFSOCK = 0o140000
        }
    )

    impl fmt::Show for SFlag {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            write!(fmt, "SFlag {{ bits: {} }}", self.bits())
        }
    }

    pub fn mknod(path: &Path, kind: SFlag, perm: FilePermission, dev: dev_t) -> SysResult<()> {
        let res = unsafe { ffi::mknod(path.to_c_str().as_ptr(), kind.bits | perm.bits(), dev) };
        from_ffi(res)
    }

    static MINORBITS: uint = 20;
    static MINORMASK: dev_t = ((1 << MINORBITS) - 1);

    pub fn mkdev(major: u64, minor: u64) -> dev_t {
        (major << MINORBITS) | minor
    }

    pub fn umask(mode: FilePermission) -> FilePermission {
        let prev = unsafe { ffi::umask(mode.bits()) };
        FilePermission::from_bits(prev).expect("[BUG] umask returned invalid FilePermission")
    }
}
