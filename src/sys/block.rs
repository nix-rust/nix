use crate::{Error, Result};
use std::fs::File;

use std::os::unix::io::AsRawFd;

/// Block device specific extensions to [`File`].
///
/// [`File`]: ../../std/fs/struct.File.html
pub trait BlckExt {
    /// Test if the file is a block device
    ///
    /// This will return `true` for a block device e.g. `"/dev/sda1"` and `false` for other files
    /// If it returns `false` using the other `BlckExt` methods on this file will almost certainly be an error.
    fn is_block_device(&self) -> bool;

    /// Get the total size of the block device in bytes.
    fn get_block_device_size(&self) -> Result<u64>;

    /// Get the size of one logical blocks in bytes.
    fn get_size_of_block(&self) -> Result<u64>;

    /// Get the number of blocks on the device.
    fn get_block_count(&self) -> Result<u64>;

    /// Ask the OS to re-read the partition table from the device.
    ///
    /// When writing an image to a block device the partions layout may change
    /// this ask the OS to re-read the partion table
    fn block_reread_paritions(&self) -> Result<()>;

    /// Does this device support zeroing on discard.
    ///
    /// Some device (e.g. SSDs with TRIM support) have the ability to mark blocks as unused in a
    /// way that means they will return zeros on future reads.
    ///
    /// If this returns `true` then all calls to [`block_discard`] will cause following reads to return zeros
    ///
    /// Some device only support zeroing on discard for certain sizes and alignements, in which case this
    /// will return `false` but some calls to [`block_discard`] may still result in zeroing some or all of the discared range.
    ///
    /// Since this is a linux only feature other systems will always return false
    ///
    /// [`block_discard`]: #tymethod.block_discard
    fn block_discard_zeros(&self) -> Result<bool>;

    /// Discard a section of the block device.
    ///
    /// Some device e.g. thinly provisioned arrays or SSDs with TRIM support have the ability to mark blocks as unused
    /// to free them up for other use. This may or maynot result in future reads to the discarded section to return
    /// zeros, see [`block_discard_zeros`] for more detail.
    ///
    /// `offset` and `length` should be given in bytes.
    ///
    /// [`block_discard_zeros`]: #tymethod.block_discard_zeros
    fn block_discard(&self, offset: u64, len: u64) -> Result<()>;

    /// Zeros out a section of the block device.
    ///
    /// There is no guaranty that there special kernel support for this and it is unlikely to be
    /// much faster that writing zeros the normal way.
    ///
    /// If there is no system call on a platfrom it will be implement by writing zeros in the normal way
    ///
    /// `offset` and `length` should be given in bytes.
    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()>;
}

#[cfg(target_os = "macos")]
impl BlckExt for File {
    fn is_block_device(&self) -> bool {
        use std::os::unix::fs::FileTypeExt;
        match self.metadata() {
            Err(_) => false,
            Ok(meta) => meta.file_type().is_block_device(),
        }
    }

    fn get_block_device_size(&self) -> Result<u64> {
        Ok(self.get_size_of_block()? * self.get_block_count()?)
    }

    fn get_size_of_block(&self) -> Result<u64> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut blksize: u32 = 0;
            ioctls::dkiocgetblocksize(fd, &mut blksize)?;
            Ok(blksize as u64)
        }
    }

    fn get_block_count(&self) -> Result<u64> {
        unsafe {
            let fd = self.as_raw_fd();
            let mut blkcount: u64 = 0;
            ioctls::dkiocgetblockcount(fd, &mut blkcount)?;
            Ok(blkcount)
        }
    }

    fn block_reread_paritions(&self) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        Ok(false)
    }

    fn block_discard(&self, offset: u64, length: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [ioctls::dk_extent { offset, length }];
        let unmap = ioctls::dk_unmap::new(&range, 0);
        unsafe {
            ioctls::dkiocunmap(fd, &unmap)?;
        }
        Ok(())
    }

    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()> {
        slow_zero(self, offset, len)
    }
}

#[allow(clippy::missing_safety_doc)]
#[cfg(target_os = "macos")]
mod ioctls {
    use crate::{ioctl_read, ioctl_write_ptr};
    use std::marker::PhantomData;
    ioctl_read!(dkiocgetblocksize, b'd', 24, u32);
    ioctl_read!(dkiocgetblockcount, b'd', 25, u64);

    #[repr(C)]
    #[derive(Copy, Clone, Debug, Default)]
    pub struct dk_extent {
        pub offset: u64,
        pub length: u64,
    }
    #[repr(C)]
    #[derive(Copy, Clone)]
    pub struct dk_unmap<'a> {
        extents: *const dk_extent,
        extents_count: u32,
        pub options: u32,
        phantom: PhantomData<&'a dk_extent>,
    }

    impl<'a> dk_unmap<'a> {
        pub fn new(extents: &'a [dk_extent], options: u32) -> dk_unmap<'a> {
            dk_unmap {
                extents: extents.as_ptr(),
                extents_count: extents.len() as u32,
                options,
                phantom: PhantomData,
            }
        }

        pub fn extents(&'a self) -> &'a [dk_extent] {
            unsafe { std::slice::from_raw_parts(self.extents, self.extents_count as usize) }
        }
    }

    impl ::std::default::Default for dk_unmap<'static> {
        fn default() -> Self {
            unsafe { ::std::mem::zeroed() }
        }
    }

    impl<'a> ::std::fmt::Debug for dk_unmap<'a> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            write!(
                f,
                "dk_unmap {{ extents: {:?}, options: {} }}",
                self.extents(),
                self.options
            )
        }
    }

    ioctl_write_ptr!(dkiocunmap, b'd', 31, dk_unmap);
}

#[cfg(target_os = "linux")]
impl BlckExt for File {
    fn is_block_device(&self) -> bool {
        use std::os::unix::fs::FileTypeExt;
        match self.metadata() {
            Err(_) => false,
            Ok(meta) => meta.file_type().is_block_device(),
        }
    }

    fn get_block_device_size(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::blkgetsize64(fd, &mut blksize)?;
            Ok(blksize)
        }
    }

    fn get_size_of_block(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::blksszget(fd, &mut blksize)?;
        }
        Ok(blksize as u64)
    }

    fn get_block_count(&self) -> Result<u64> {
        Ok(self.get_block_device_size()? / self.get_size_of_block()?)
    }

    fn block_reread_paritions(&self) -> Result<()> {
        let fd = self.as_raw_fd();
        unsafe {
            ioctls::blkrrpart(fd)?;
        }
        Ok(())
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        let fd = self.as_raw_fd();
        let mut discard_zeros = 0;
        unsafe {
            ioctls::blkdiscardzeros(fd, &mut discard_zeros)?;
        }
        Ok(discard_zeros != 0)
    }

    fn block_discard(&self, offset: u64, len: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [offset, len];
        unsafe {
            ioctls::blkdiscard(fd, &range)?;
        }
        Ok(())
    }

    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()> {
        let fd = self.as_raw_fd();
        let range = [offset, len];
        unsafe {
            ioctls::blkzeroout(fd, &range)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod ioctls {
    use crate::*;

    ioctl_none!(blkrrpart, 0x12, 95);
    ioctl_read_bad!(
        blkgetsize64,
        request_code_read!(0x12, 114, ::std::mem::size_of::<usize>()),
        u64
    );
    ioctl_read_bad!(
        blkdiscardzeros,
        request_code_none!(0x12, 124),
        ::std::os::raw::c_uint
    );
    ioctl_write_ptr_bad!(blkdiscard, request_code_none!(0x12, 119), [u64; 2]);
    ioctl_write_ptr_bad!(blkzeroout, request_code_none!(0x12, 127), [u64; 2]);
    ioctl_read_bad!(
        blksszget,
        request_code_none!(0x12, 104),
        ::std::os::raw::c_int
    );
}

#[cfg(target_os = "freebsd")]
impl BlckExt for File {
    fn is_block_device(&self) -> bool {
        // free BSD does not support "block" devices, so instead check if file is a disk
        // style device by asking for the block size
        // https://www.freebsd.org/doc/en/books/arch-handbook/driverbasics-block.html
        self.get_size_of_block().is_ok()
    }

    fn get_block_device_size(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::diocgmediasize(fd, &mut blksize)?;
            Ok(blksize as u64)
        }
    }

    fn get_size_of_block(&self) -> Result<u64> {
        let fd = self.as_raw_fd();
        let mut blksize = 0;
        unsafe {
            ioctls::diocgsectorsize(fd, &mut blksize)?;
        }
        Ok(blksize as u64)
    }

    fn get_block_count(&self) -> Result<u64> {
        Ok(self.get_block_device_size()? / self.get_size_of_block()?)
    }

    fn block_reread_paritions(&self) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn block_discard_zeros(&self) -> Result<bool> {
        Ok(false)
    }
    fn block_discard(&self, _offset: u64, _len: u64) -> Result<()> {
        Err(Error::UnsupportedOperation)
    }

    fn block_zero_out(&mut self, offset: u64, len: u64) -> Result<()> {
        slow_zero(self, offset, len)
    }
}

#[cfg(target_os = "freebsd")]
pub mod ioctls {
    use crate::*;

    ioctl_read!(diocgmediasize, b'd', 129, libc::off_t);
    ioctl_read!(diocgsectorsize, b'd', 128, ::std::os::raw::c_uint);
}

#[cfg(any(target_os = "freebsd", target_os = "macos"))]
fn slow_zero(file: &mut File, offset: u64, len: u64) -> Result<()> {
    use std::io::{Seek, SeekFrom, Write};
    const BUF_SIZE: usize = 1024;
    let zeros = [0; BUF_SIZE];
    let oldpos = file.seek(SeekFrom::Start(offset))?;
    let mut remaining = len;
    while remaining > BUF_SIZE as u64 {
        file.write_all(&zeros)?;
        remaining -= BUF_SIZE as u64;
    }
    file.write_all(&zeros[0..remaining as usize])?;
    file.seek(SeekFrom::Start(oldpos))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    const DEV: &str = "/dev/sda";
    #[cfg(target_os = "freebsd")]
    const DEV: &str = "/dev/nvme0ns1";
    #[cfg(target_os = "macos")]
    const DEV: &str = "/dev/disk2";
    use super::*;
    #[test]
    fn get_block_device_size_returns_bytes() -> () {
        let gb = 1 << 30;
        let file = File::open(DEV).unwrap();
        let bytes = file.get_block_device_size().unwrap();
        println!("disk is {} blocks {}gb", bytes, bytes / gb);
        assert!(bytes > 1 * gb);
    }

    #[test]
    fn is_block_device_return_true() -> () {
        let file = File::open(DEV).unwrap();
        let is_block = file.is_block_device();
        println!("disk is block?  {}", is_block);
        assert!(is_block);
    }

    #[test]
    fn get_size_of_block_returns_power_of_two() -> () {
        let file = File::open(DEV).unwrap();
        let bytes = file.get_size_of_block().unwrap();
        println!("block is {}", bytes);
        assert!(bytes > 400);
        assert_eq!(bytes & (bytes - 1), 0);
    }
}
