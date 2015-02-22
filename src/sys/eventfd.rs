use std::mem;
use libc::{c_int, c_uint};
use errno::Errno;
use fcntl::Fd;
use {Error, Result};

bitflags!(
    flags EventFdFlag: c_int {
        const EFD_CLOEXEC   = 0o2000000, // Since Linux 2.6.27
        const EFD_NONBLOCK  = 0o0004000, // Since Linux 2.6.27
        const EFD_SEMAPHORE = 0o0000001, // Since Linux 2.6.30
    }
);

pub fn eventfd(initval: usize, flags: EventFdFlag) -> Result<Fd> {
    type F = unsafe extern "C" fn(initval: c_uint, flags: c_int) -> c_int;

    extern {
        #[linkage = "extern_weak"]
        static eventfd: *const ();
    }

    if eventfd.is_null() {
        panic!("eventfd unsupported on this platform");
    }

    let res = unsafe {
        mem::transmute::<*const (), F>(eventfd)(
            initval as c_uint, flags.bits())
    };

    if res < 0 {
        return Err(Error::Sys(Errno::last()));
    }

    Ok(res)
}
