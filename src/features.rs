pub use self::os::*;

#[cfg(target_os = "linux")]
mod os {
    use sys::utsname::uname;

    // Features:
    // * atomic cloexec on socket: 2.6.27
    // * pipe2: 2.6.27
    // * accept4: 2.6.28

    static VERS_UNKNOWN: uint = 1;
    static VERS_2_6_18:  uint = 2;
    static VERS_2_6_27:  uint = 3;
    static VERS_2_6_28:  uint = 4;
    static VERS_3:       uint = 5;

    fn parse_kernel_version() -> uint {
        let u = uname();

        #[inline]
        fn digit(dst: &mut uint, b: u8) {
            *dst *= 10;
            *dst += (b - b'0') as uint;
        }

        let mut curr = 0u;
        let mut major = 0;
        let mut minor = 0;
        let mut patch = 0;

        for b in u.release().bytes() {
            if curr >= 3 {
                break;
            }

            match b {
                b'.' | b'-' => {
                    curr += 1;
                }
                b'0'...b'9' => {
                    match curr {
                        0 => digit(&mut major, b),
                        1 => digit(&mut minor, b),
                        _ => digit(&mut patch, b),
                    }
                }
                _ => break,
            }
        }

        if major >= 3 {
            VERS_3
        } else if major >= 2 {
            if minor >= 7 {
                VERS_UNKNOWN
            } else if minor >= 6 {
                if patch >= 28 {
                    VERS_2_6_28
                } else if patch >= 27 {
                    VERS_2_6_27
                } else {
                    VERS_2_6_18
                }
            } else {
                VERS_UNKNOWN
            }
        } else {
            VERS_UNKNOWN
        }
    }

    fn kernel_version() -> uint {
        static mut KERNEL_VERS: uint = 0;

        unsafe {
            if KERNEL_VERS == 0 {
                KERNEL_VERS = parse_kernel_version();
            }

            KERNEL_VERS
        }
    }

    pub fn socket_atomic_cloexec() -> bool {
        kernel_version() >= VERS_2_6_27
    }

    #[test]
    pub fn test_parsing_kernel_version() {
        assert!(kernel_version() > 0);
    }
}

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod os {
    pub fn socket_atomic_cloexec() -> bool {
        false
    }
}
