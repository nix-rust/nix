use nix::Error;
use nix::libc::{c_void, size_t};
use nix::sys::mman::{mmap, MapFlags, ProtFlags, mincore, mincore_vec_char_t};

#[cfg(target_os = "linux")]
use nix::sys::mman::{mremap, MRemapFlags};

#[test]
fn test_mmap_anonymous() {
    let ref mut byte = unsafe {
        let ptr = mmap(std::ptr::null_mut(), 1,
                       ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                       MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS, -1, 0)
                      .unwrap();
        *(ptr as * mut u8)
    };
    assert_eq !(*byte, 0x00u8);
    *byte = 0xffu8;
    assert_eq !(*byte, 0xffu8);
}

#[test]
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
fn test_mremap_grow() {
    const ONE_K : size_t = 1024;
    let slice : &mut[u8] = unsafe {
        let mem = mmap(std::ptr::null_mut(), ONE_K,
                       ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                       MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE, -1, 0)
                      .unwrap();
        std::slice::from_raw_parts_mut(mem as * mut u8, ONE_K)
    };
    assert_eq !(slice[ONE_K - 1], 0x00);
    slice[ONE_K - 1] = 0xFF;
    assert_eq !(slice[ONE_K - 1], 0xFF);

    let slice : &mut[u8] = unsafe {
        #[cfg(target_os = "linux")]
        let mem = mremap(slice.as_mut_ptr() as * mut c_void, ONE_K, 10 * ONE_K,
                         MRemapFlags::MREMAP_MAYMOVE, None)
                      .unwrap();
        #[cfg(target_os = "netbsd")]
        let mem = mremap(slice.as_mut_ptr() as * mut c_void, ONE_K, 10 * ONE_K,
                         MRemapFlags::MAP_REMAPDUP, None)
                      .unwrap();
        std::slice::from_raw_parts_mut(mem as * mut u8, 10 * ONE_K)
    };

    // The first KB should still have the old data in it.
    assert_eq !(slice[ONE_K - 1], 0xFF);

    // The additional range should be zero-init'd and accessible.
    assert_eq !(slice[10 * ONE_K - 1], 0x00);
    slice[10 * ONE_K - 1] = 0xFF;
    assert_eq !(slice[10 * ONE_K - 1], 0xFF);
}

#[test]
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
fn test_mremap_shrink() {
    const ONE_K : size_t = 1024;
    let slice : &mut[u8] = unsafe {
        let mem = mmap(std::ptr::null_mut(), 10 * ONE_K,
                       ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                       MapFlags::MAP_ANONYMOUS | MapFlags::MAP_PRIVATE, -1, 0)
                      .unwrap();
        std::slice::from_raw_parts_mut(mem as * mut u8, ONE_K)
    };
    assert_eq !(slice[ONE_K - 1], 0x00);
    slice[ONE_K - 1] = 0xFF;
    assert_eq !(slice[ONE_K - 1], 0xFF);

    let slice : &mut[u8] = unsafe {
        #[cfg(target_os = "linux")]
        let mem = mremap(slice.as_mut_ptr() as * mut c_void, 10 * ONE_K, ONE_K,
                         MRemapFlags::empty(), None)
                      .unwrap();
        // Since we didn't supply MREMAP_MAYMOVE, the address should be the
        // same.
        #[cfg(target_os = "linux")]
        let mem = mremap(slice.as_mut_ptr() as * mut c_void, 10 * ONE_K, ONE_K,
                         MRemapFlags::MAP_FIXED), None)
                      .unwrap();
        assert_eq !(mem, slice.as_mut_ptr() as * mut c_void);
        std::slice::from_raw_parts_mut(mem as * mut u8, ONE_K)
    };

    // The first KB should still be accessible and have the old data in it.
    assert_eq !(slice[ONE_K - 1], 0xFF);
}

#[test]
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "macos"))]
fn test_mincore() {
    use std::os::unix::io::AsRawFd;

    let page_size = nix::unistd::sysconf(nix::unistd::SysconfVar::PAGE_SIZE)
        .ok()
        .flatten()
        .unwrap() as usize;
    let file = std::fs::File::open("/etc/hosts").unwrap();
    let file_meta = file.metadata().unwrap();
    let file_len = file_meta.len() as usize;
    let mem = unsafe {
        mmap(std::ptr::null_mut(),
             file_meta.len() as usize,
             ProtFlags::PROT_READ,
             MapFlags::MAP_PRIVATE, file.as_raw_fd(), 0)
    }.unwrap();

    let pages = (file_len + page_size + 1) / page_size;
    let mincore_vec = unsafe {
        libc::malloc(pages)
    } as *mut mincore_vec_char_t;

    assert!(mincore(mem, file_len, mincore_vec).is_ok());
}
