//! Provide helpers for making ioctl system calls
//!
//! # Overview of IOCTLs
//!
//! The `ioctl` system call is a widely support system
//! call on *nix systems providing access to functions
//! and data that do not fit nicely into the standard
//! read and write operations on a file itself.  It is
//! common to see ioctls used for the following purposes:
//!
//! * Provide read/write access to out-of-band data related
//!   to a device such as configuration (for instance, setting
//!   serial port options)
//! * Provide a mechanism for performing full-duplex data
//!   transfers (for instance, xfer on SPI devices).
//! * Provide access to control functions on a device (for example,
//!   on Linux you can send commands like pause, resume, and eject
//!   to the CDROM device.
//! * Do whatever else the device driver creator thought made most sense.
//!
//! Ioctls are synchronous system calls and are similar to read and
//! write calls in that regard.
//!
//! The prototype for the ioctl system call in libc is as follows:
//!
//! ```c
//! int ioctl(int fd, unsigned long request, ...);
//! ```
//!
//! Typically, an ioctl takes 3 parameters as arguments:
//!
//! 1. An open file descriptor, `fd`.
//! 2. An device-dependennt request code or operation.  This request
//!    code is referred to as `op` in this module.
//! 3. Either a pointer to a location in memory or an integer.  This
//!    number of pointer may either be used by the kernel or written
//!    to by the kernel depending on how the operation is documented
//!    to work.
//!
//! The `op` request code is essentially an arbitrary integer having
//! a device-driver specific meaning.  Over time, it proved difficult
//! for various driver implementors to use this field sanely, so a
//! convention with macros was introduced to the Linux Kernel that
//! is used by most newer drivers.  See
//! https://github.com/torvalds/linux/blob/master/Documentation/ioctl/ioctl-number.txt
//! for additional details.  The macros exposed by the kernel for
//! consumers are implemented in this module and may be used to
//! instead of calls like `_IOC`, `_IO`, `_IOR`, and `_IOW`.
//!
//! # Interface Overview
//!
//! This ioctl module seeks to tame the ioctl beast by providing
//! a set of safer (although not safe) functions
//! implementing the most common ioctl access patterns.
//!
//! The most common access patterns for ioctls are as follows:
//!
//! 1. `read`: A pointer is provided to the kernel which is populated
//!    with a value containing the "result" of the operation.  The
//!    result may be an integer or structure.  The kernel may also
//!    read values from the provided pointer (usually a structure).
//! 2. `write`: A pointer is provided to the kernel containing values
//!    that the kernel will read in order to perform the operation.
//! 3. `execute`: The operation is passed to the kernel but no
//!    additional pointer is passed.  The operation is enough
//!    and it either succeeds or results in an error.
//!
//! Where appropriate, versions of these interface function are provided
//! taking either refernces or pointers.  The pointer versions are
//! necessary for cases (notably slices) where a reference cannot
//! be generically cast to a pointer.

use {Error, Result, errno};
use libc::{c_int, c_ulong};
use libc::funcs::bsd44::ioctl as libc_ioctl;
use std::mem;
use std::os::unix::io::RawFd;

pub type ioctl_op_t = c_ulong;

// low-level ioctl functions and definitions matching the
// macros provided in ioctl.h from the kernel
const IOC_NRBITS: u32 = 8;
const IOC_TYPEBITS: u32 = 8;
const IOC_SIZEBITS: u32 = 14;
// const IOC_DIRBITS: u32 = 2;

const IOC_NRSHIFT: u32 = 0;
const IOC_TYPESHIFT: u32 = IOC_NRSHIFT + IOC_NRBITS;
const IOC_SIZESHIFT: u32 = IOC_TYPESHIFT + IOC_TYPEBITS;
const IOC_DIRSHIFT: u32 = IOC_SIZESHIFT + IOC_SIZEBITS;

/// Flags indicating the direction of the ioctl operation
/// for ioctls using modern operation conventions
bitflags! {
    flags IoctlDirFlags: u8 {
        /// Indicates that the ioctl data pointer is not used
        const IOC_NONE  = 0x00,
        /// Indicates that the ioctl data pointer contains data that
        /// will be consumed by the operating system
        const IOC_WRITE = 0x01,
        /// Indicates tha the ioctl data pointer contains data that
        /// will be populated by the operating system to be consumed
        /// by userspace
        const IOC_READ  = 0x02,
    }
}

/// Build an ioctl op with the provide parameters.  This is a helper
/// function for IOCTLs in the Linux kernel using the newer conventions
/// for IOCTLs operations.  Many ioctls do not use this newer convention
/// and the constants for those should just be used as-is.
///
/// This provides the same functionality as the Linux `_IOC` macro.
pub fn op(dir: IoctlDirFlags, ioctl_type: u8, nr: u8, size: usize) -> ioctl_op_t {
    // actual number will always fit in 32 bits, but ioctl() expects
    // an unsigned long for the op
    let size_to_use: u32 = if size < (1 << IOC_SIZEBITS) { size as u32 } else { 0 };
    (((dir.bits as u32) << IOC_DIRSHIFT) |
     ((ioctl_type as u32) << IOC_TYPESHIFT) |
     ((nr as u32) << IOC_NRSHIFT) |
     ((size_to_use) << IOC_SIZESHIFT)) as ioctl_op_t
}

/// Build an op indicating that the data pointer is not used.
/// That is, the command itself is sufficient.
///
/// This provides the same functionality the Linux `_IO` macro.
pub fn op_none(ioctl_type: u8, nr: u8) -> ioctl_op_t {
    op(IOC_NONE, ioctl_type, nr, 0)
}

/// Build an op indicating that the data pointer will be populated
/// with data from the kernel
///
/// This provides the same functionality as the Linux `_IOR` macro.
pub fn op_read(ioctl_type: u8, nr: u8, size: usize) -> ioctl_op_t {
    op(IOC_READ, ioctl_type, nr, size)
}

/// Build an op indicating that the data pointer contains data
/// to be consumed by the kernel (and not written to).
///
/// This provides the same functionality as the Linux `_IOW` macro.
pub fn op_write(ioctl_type: u8, nr: u8, size: usize) -> ioctl_op_t {
    op(IOC_WRITE, ioctl_type, nr, size)
}

/// Build an op indicating that the data pointer both contains
/// data to be consumed by the kernel and contains fields that
/// will be populated by the kernel.
///
/// This provides the same functionality as the Linux `_IOWR` macro.
pub fn op_read_write(ioctl_type: u8, nr: u8, size: usize) -> ioctl_op_t {
    op(IOC_WRITE | IOC_READ, ioctl_type, nr, size)
}

fn convert_ioctl_res(res: c_int) -> Result<c_int> {
    if res < 0 {
        return Err(Error::Sys(errno::Errno::last()))
    }
    Ok(res) // res may length or similar useful to caller
}

/// Ioctl call that is expected to return a result
/// but which does not take any additional arguments on the input side
///
/// This function will allocate allocate space for and returned an owned
/// reference to the result.
pub unsafe fn read<T>(fd: RawFd, op: ioctl_op_t) -> Result<T> {
    // allocate memory for the result (should get a value from kernel)
    let mut dst: T = mem::zeroed();
    let dst_ptr: *mut T = &mut dst;
    try!(read_into_ptr(fd, op, dst_ptr));
    Ok(dst)
}

/// Ioctl where the result from the kernel will be written to the
/// provided reference
///
/// The refernced data may also contain information that will be consumed
/// by the kernel.
pub unsafe fn read_into<T>(fd: RawFd, op: ioctl_op_t, data: &mut T) -> Result<c_int> {
    read_into_ptr(fd, op, data as *mut T)
}

/// Ioctl where the result from the kernel will be written to the
/// provided pointer
///
/// The refernced data may also contain information that will be consumed
/// by the kernel.
pub unsafe fn read_into_ptr<T>(fd: RawFd, op: ioctl_op_t, data_ptr: *mut T) -> Result<c_int> {
    convert_ioctl_res(libc_ioctl(fd, op, data_ptr))
}

/// Ioctl call that sends a value to the kernel but
/// does not return anything (pure side effect).
pub unsafe fn write<T>(fd: RawFd, op: ioctl_op_t, data: &T) -> Result<c_int> {
    write_ptr(fd, op, data as *const T)
}

/// Ioctl call that sends a value to the kernel but
/// does not return anything (pure side effect).
pub unsafe fn write_ptr<T>(fd: RawFd, op: ioctl_op_t, data: *const T) -> Result<c_int> {
    convert_ioctl_res(libc_ioctl(fd, op, data as *const T))
}

/// Ioctl call for which no data pointer is provided to the kernel.
/// That is, the kernel has sufficient information about what to
/// do based on the op alone.
pub fn execute(fd: RawFd, op: ioctl_op_t) -> Result<c_int> {
    convert_ioctl_res(unsafe { libc_ioctl(fd, op) })
}
