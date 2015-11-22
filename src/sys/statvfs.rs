//! FFI for statvfs functions
//!
//! See the `vfs::Statvfs` struct for some rusty wrappers

use {Result, NixPath, from_ffi};
use errno::Errno;
use std::os::unix::io::AsRawFd;

pub mod vfs {
	//! Structs related to the `statvfs` and `fstatvfs` functions
	//!
	//! The `Statvfs` struct has some wrappers methods around the `statvfs` and
	//! `fstatvfs` calls.

	use libc::{c_ulong,c_int};
	use std::os::unix::io::AsRawFd;
	use {Result, NixPath};

	use super::{statvfs, fstatvfs};

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
	#[derive(Debug,Default,Copy,Clone)]
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

	impl Statvfs {
		/// Create a new `Statvfs` object and fill it with information about
		/// the mount that contains `path`
		pub fn for_path<P: ?Sized + NixPath>(path: &P) -> Result<Statvfs> {
			let mut stat = Statvfs::default();
			let res = statvfs(path, &mut stat);
			res.map(|_| stat)
		}

		/// Replace information in this struct with information about `path`
		pub fn update_with_path<P: ?Sized + NixPath>(&mut self, path: &P) -> Result<()> {
			statvfs(path, self)
		}

		/// Create a new `Statvfs` object and fill it with information from fd
		pub fn for_fd<T: AsRawFd>(fd: &T) -> Result<Statvfs> {
			let mut stat = Statvfs::default();
			let res = fstatvfs(fd, &mut stat);
			res.map(|_| stat)
		}

		/// Replace information in this struct with information about `fd`
		pub fn update_with_fd<T: AsRawFd>(&mut self, fd: &T) -> Result<()> {
			fstatvfs(fd, self)
		}
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
