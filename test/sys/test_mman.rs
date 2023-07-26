use nix::sys::mman::{mmap, MapFlags, ProtFlags};
use nix::unistd::{lseek, sysconf, write, SysconfVar, Whence};
use std::num::NonZeroUsize;
use std::os::unix::io::{AsRawFd, BorrowedFd};
use tempfile::tempfile;

use crate::{require_largefile, skip};

#[test]
fn test_mmap_anonymous() {
    unsafe {
        let ptr = mmap::<BorrowedFd>(
            None,
            NonZeroUsize::new(1).unwrap(),
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS,
            None,
            0,
        )
        .unwrap() as *mut u8;
        assert_eq!(*ptr, 0x00u8);
        *ptr = 0xffu8;
        assert_eq!(*ptr, 0xffu8);
    }
}

#[test]
fn test_mmap_largefile() {
    require_largefile!("test_mmap_largefile");

    // Calculate an offset that requires more than 32 bits to represent
    // and is a multiple of the page size.
    #[allow(clippy::unnecessary_cast)] // Some platforms need the cast.
    let pagesize = match sysconf(SysconfVar::PAGE_SIZE) {
        Ok(Some(x)) => x,
        _ => {
            skip!("test_mmap_largefile: Could not determine page size. Skipping test.");
        }
    } as i64;
    let pages_in_32bits = 1_0000_0000i64 / pagesize;
    // Silence Clippy warning - offset's type varies by platform.
    #[allow(clippy::useless_conversion)]
    let offset = ((pages_in_32bits + 2) * pagesize).try_into().unwrap();
    // Create a file, seek to offset, and write some bytes.
    let file = tempfile().unwrap();
    lseek(file.as_raw_fd(), offset, Whence::SeekSet).unwrap();
    write(file.as_raw_fd(), b"xyzzy").unwrap();
    // Check that we can map it and see the bytes.
    let slice = unsafe {
        let res = mmap(
            None,
            NonZeroUsize::new(5).unwrap(),
            ProtFlags::PROT_READ,
            MapFlags::MAP_PRIVATE,
            Some(&file),
            offset,
        );
        assert!(res.is_ok());
        std::slice::from_raw_parts(res.unwrap() as *const u8, 5)
    };
    assert_eq!(slice, b"xyzzy");
}

#[test]
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
fn test_mremap_grow() {
    use nix::libc::{c_void, size_t};
    use nix::sys::mman::{mremap, MRemapFlags};

    const ONE_K: size_t = 1024;
    let one_k_non_zero = NonZeroUsize::new(ONE_K).unwrap();

    let slice: &mut [u8] = unsafe {
        let mem = mmap::<BorrowedFd>(
            None,
            one_k_non_zero,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE,
            None,
            0,
        )
        .unwrap();
        std::slice::from_raw_parts_mut(mem as *mut u8, ONE_K)
    };
    assert_eq!(slice[ONE_K - 1], 0x00);
    slice[ONE_K - 1] = 0xFF;
    assert_eq!(slice[ONE_K - 1], 0xFF);

    let slice: &mut [u8] = unsafe {
        #[cfg(target_os = "linux")]
        let mem = mremap(
            slice.as_mut_ptr() as *mut c_void,
            ONE_K,
            10 * ONE_K,
            MRemapFlags::MREMAP_MAYMOVE,
            None,
        )
        .unwrap();
        #[cfg(target_os = "netbsd")]
        let mem = mremap(
            slice.as_mut_ptr() as *mut c_void,
            ONE_K,
            10 * ONE_K,
            MRemapFlags::MAP_REMAPDUP,
            None,
        )
        .unwrap();
        std::slice::from_raw_parts_mut(mem as *mut u8, 10 * ONE_K)
    };

    // The first KB should still have the old data in it.
    assert_eq!(slice[ONE_K - 1], 0xFF);

    // The additional range should be zero-init'd and accessible.
    assert_eq!(slice[10 * ONE_K - 1], 0x00);
    slice[10 * ONE_K - 1] = 0xFF;
    assert_eq!(slice[10 * ONE_K - 1], 0xFF);
}

#[test]
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
// Segfaults for unknown reasons under QEMU for 32-bit targets
#[cfg_attr(all(target_pointer_width = "32", qemu), ignore)]
fn test_mremap_shrink() {
    use nix::libc::{c_void, size_t};
    use nix::sys::mman::{mremap, MRemapFlags};
    use std::num::NonZeroUsize;

    const ONE_K: size_t = 1024;
    let ten_one_k = NonZeroUsize::new(10 * ONE_K).unwrap();
    let slice: &mut [u8] = unsafe {
        let mem = mmap::<BorrowedFd>(
            None,
            ten_one_k,
            ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
            MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE,
            None,
            0,
        )
        .unwrap();
        std::slice::from_raw_parts_mut(mem as *mut u8, ONE_K)
    };
    assert_eq!(slice[ONE_K - 1], 0x00);
    slice[ONE_K - 1] = 0xFF;
    assert_eq!(slice[ONE_K - 1], 0xFF);

    let slice: &mut [u8] = unsafe {
        let mem = mremap(
            slice.as_mut_ptr() as *mut c_void,
            ten_one_k.into(),
            ONE_K,
            MRemapFlags::empty(),
            None,
        )
        .unwrap();
        // Since we didn't supply MREMAP_MAYMOVE, the address should be the
        // same.
        assert_eq!(mem, slice.as_mut_ptr() as *mut c_void);
        std::slice::from_raw_parts_mut(mem as *mut u8, ONE_K)
    };

    // The first KB should still be accessible and have the old data in it.
    assert_eq!(slice[ONE_K - 1], 0xFF);
}
