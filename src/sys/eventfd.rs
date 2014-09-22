use libc::{c_int, c_uint};
use fcntl::Fd;
use errno::{SysResult, SysError, from_ffi};

mod ffi {
    use libc::{c_int, c_uint};

    extern {
        pub fn eventfd(initval: c_uint, flags: c_int) -> c_int;
    }
}

bitflags!(
    flags EventFdFlag: c_int {
        static EFD_CLOEXEC   = 0o2000000, // Since Linux 2.6.27
        static EFD_NONBLOCK  = 0o0004000, // Since Linux 2.6.27
        static EFD_SEMAPHORE = 0o0000001, // Since Linux 2.6.30
    }
)

pub fn eventfd(initval: uint, flags: EventFdFlag) -> SysResult<Fd> {
    let res = unsafe { ffi::eventfd(initval as c_uint, flags.bits()) };

    if res < 0 {
        return Err(SysError::last());
    }

    Ok(res)
}
