pub trait AsRawFd {
    fn as_raw_fd(&self) -> RawFd;
}

pub trait FromRawFd {
    unsafe fn from_raw_fd(fd: RawFd) -> OwnedFd;
}

pub trait IntoRawFd {
    fn into_raw_fd(self) -> RawFd;
}

pub trait AsFd {
    fn as_fd(&self) -> BorrowedFd<'_>;
}

#[repr(transparent)]
pub struct OwnedFd {
    fd: RawFd,
}

impl Drop for OwnedFd {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            // Note that errors are ignored when closing a file descriptor. The
            // reason for this is that if an error occurs we don't actually know if
            // the file descriptor was closed or not, and if we retried (for
            // something like EINTR), we might close another valid file descriptor
            // opened after we closed ours.
            #[cfg(not(target_os = "hermit"))]
            let _ = libc::close(self.fd);
            #[cfg(target_os = "hermit")]
            let _ = hermit_abi::close(self.fd);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BorrowedFd<'fd> {
    fd: RawFd,
    _phantom: core::marker::PhantomData<&'fd OwnedFd>,
}

pub type RawFd = core::ffi::c_int;
