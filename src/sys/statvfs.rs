use {Result, NixPath, from_ffi};
use errno::Errno;
use std::os::unix::io::AsRawFd;

pub mod vfs {
	use libc::{c_ulong,c_int};

	bitflags!(
		#[repr(C)]
		#[derive(Default)]
		flags FsFlags: c_ulong {
			const RDONLY = 1,
			const NOSUID = 2,
			const NODEV  = 4,
			const NOEXEC = 8,
			const SYNCHRONOUS  = 16,
			const MANDLOCK = 64,
			const WRITE = 128,
			const APPEND = 256,
			const IMMUTABLE = 512,
			const NOATIME = 1024,
			const NODIRATIME = 2048,
			const RELATIME = 4096,
		}
	);

	#[repr(C)]
	#[derive(Debug,Copy,Clone)]
	pub struct Statvfs {
		pub f_bsize: c_ulong,
		pub f_frsize: c_ulong,
		pub f_blocks: u64,
		pub f_bfree: u64,
		pub f_bavail: u64,
		pub f_files: u64,
		pub f_ffree: u64,
		pub f_favail: u64,
		pub f_fsid: c_ulong,
		pub f_flag: FsFlags,
		pub f_namemax: c_ulong,
		f_spare: [c_int; 6],
	}
}

mod ffi {
	use libc::{c_char, c_int};
	use sys::statvfs::vfs;

	extern {
		pub fn statvfs(path: * const c_char, buf: *mut vfs::Statvfs) -> c_int;
		pub fn fstatvfs(fd: c_int, buf: *mut vfs::Statvfs) -> c_int;
	}
}

pub fn statvfs<P: ?Sized + NixPath>(path: &P, stat: &mut vfs::Statvfs) -> Result<()> {
	unsafe {
		Errno::clear();
		let res = try!(
			path.with_nix_path(|path| ffi::statvfs(path.as_ptr(), stat))
		);
		from_ffi(res)
	}
}

pub fn fstatvfs<T: AsRawFd>(fd: &T, stat: &mut vfs::Statvfs) -> Result<()> {
	unsafe {
		Errno::clear();
		from_ffi(ffi::fstatvfs(fd.as_raw_fd(), stat))
	}
}
