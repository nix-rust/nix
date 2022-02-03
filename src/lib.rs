//! Rust friendly bindings to the various *nix system functions.
//!
//! Modules are structured according to the C header file that they would be
//! defined in.
//!
//! # Features
//!
//! Nix uses the following Cargo features to enable optional functionality.
//! They may be enabled in any combination.
//! * `acct` - Process accounting
//! * `aio` - POSIX AIO
//! * `dir` - Stuff relating to directory iteration
//! * `env` - Manipulate environment variables
//! * `event` - Event-driven APIs, like `kqueue` and `epoll`
//! * `features` - Query characteristics of the OS at runtime
//! * `fs` - File system functionality
//! * `hostname` - Get and set the system's hostname
//! * `inotify` - Linux's `inotify` file system notification API
//! * `ioctl` - The `ioctl` syscall, and wrappers for my specific instances
//! * `kmod` - Load and unload kernel modules
//! * `mman` - Stuff relating to memory management
//! * `mount` - Mount and unmount file systems
//! * `mqueue` - POSIX message queues
//! * `net` - Networking-related functionality
//! * `personality` - Set the process execution domain
//! * `poll` - APIs like `poll` and `select`
//! * `process` - Stuff relating to running processes
//! * `pthread` - POSIX threads
//! * `ptrace` - Process tracing and debugging
//! * `quota` - File system quotas
//! * `reboot` - Reboot the system
//! * `resource` - Process resource limits
//! * `sched` - Manipulate process's scheduling
//! * `signal` - Send and receive signals to processes
//! * `term` - Terminal control APIs
//! * `time` - Query the operating system's clocks
//! * `ucontext` - User thread context
//! * `uio` - Vectored I/O
//! * `users` - Stuff relating to users and groups
//! * `zerocopy` - APIs like `sendfile` and `copy_file_range`
#![crate_name = "nix"]
#![cfg(unix)]
#![cfg_attr(docsrs, doc(cfg(all())))]
#![allow(non_camel_case_types)]
#![cfg_attr(test, deny(warnings))]
#![recursion_limit = "500"]
#![deny(unused)]
#![allow(unused_macros)]
#![cfg_attr(not(feature = "default"), allow(unused_imports))]
#![deny(unstable_features)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-exported external crates
pub use libc;

// Private internal modules
#[macro_use]
mod macros;

// Public crates
#[cfg(not(target_os = "redox"))]
feature! {
    #![feature = "dir"]
    #[allow(missing_docs)]
    pub mod dir;
}
feature! {
    #![feature = "env"]
    pub mod env;
}
#[allow(missing_docs)]
pub mod errno;
feature! {
    #![feature = "features"]

    #[deny(missing_docs)]
    pub mod features;
}
#[allow(missing_docs)]
pub mod fcntl;
feature! {
    #![feature = "net"]

    #[cfg(any(target_os = "android",
              target_os = "dragonfly",
              target_os = "freebsd",
              target_os = "ios",
              target_os = "linux",
              target_os = "macos",
              target_os = "netbsd",
              target_os = "illumos",
              target_os = "openbsd"))]
    #[deny(missing_docs)]
    pub mod ifaddrs;
    #[cfg(not(target_os = "redox"))]
    #[deny(missing_docs)]
    pub mod net;
}
#[cfg(any(target_os = "android", target_os = "linux"))]
feature! {
    #![feature = "kmod"]
    #[allow(missing_docs)]
    pub mod kmod;
}
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
feature! {
    #![feature = "mount"]
    pub mod mount;
}
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fushsia",
    target_os = "linux",
    target_os = "netbsd"
))]
feature! {
    #![feature = "mqueue"]
    #[allow(missing_docs)]
    pub mod mqueue;
}
feature! {
    #![feature = "poll"]
    pub mod poll;
}
#[cfg(not(any(target_os = "redox", target_os = "fuchsia")))]
feature! {
    #![feature = "term"]
    #[deny(missing_docs)]
    pub mod pty;
}
feature! {
    #![feature = "sched"]
    pub mod sched;
}
pub mod sys;
feature! {
    #![feature = "time"]
    #[allow(missing_docs)]
    pub mod time;
}
// This can be implemented for other platforms as soon as libc
// provides bindings for them.
#[cfg(all(target_os = "linux", any(target_arch = "x86", target_arch = "x86_64")))]
feature! {
    #![feature = "ucontext"]
    #[allow(missing_docs)]
    pub mod ucontext;
}
#[allow(missing_docs)]
pub mod unistd;

/*
 *
 * ===== Result / Error =====
 *
 */

use libc::PATH_MAX;

use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::result;

use errno::Errno;

/// Nix Result Type
pub type Result<T> = result::Result<T, Errno>;

/// Nix's main error type.
///
/// It's a wrapper around Errno.  As such, it's very interoperable with
/// [`std::io::Error`], but it has the advantages of:
/// * `Clone`
/// * `Copy`
/// * `Eq`
/// * Small size
/// * Represents all of the system's errnos, instead of just the most common
/// ones.
pub type Error = Errno;

/// Common trait used to represent file system paths by many Nix functions.
pub trait NixPath {
    /// Is the path empty?
    fn is_empty(&self) -> bool;

    /// Length of the path in bytes
    fn len(&self) -> usize;

    /// Execute a function with this path as a `CStr`.
    ///
    /// Mostly used internally by Nix.
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T;
}

impl NixPath for str {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(OsStr::new(self))
    }

    fn len(&self) -> usize {
        NixPath::len(OsStr::new(self))
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        OsStr::new(self).with_nix_path(f)
    }
}

impl NixPath for OsStr {
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_bytes().with_nix_path(f)
    }
}

impl NixPath for CStr {
    fn is_empty(&self) -> bool {
        self.to_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.to_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        // Equivalence with the [u8] impl.
        if self.len() >= PATH_MAX as usize {
            return Err(Errno::ENAMETOOLONG);
        }

        Ok(f(self))
    }
}

impl NixPath for [u8] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(Errno::ENAMETOOLONG);
        }

        buf[..self.len()].copy_from_slice(self);
        match CStr::from_bytes_with_nul(&buf[..=self.len()]) {
            Ok(s) => Ok(f(s)),
            Err(_) => Err(Errno::EINVAL),
        }
    }
}

impl NixPath for Path {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_os_str().with_nix_path(f)
    }
}

impl NixPath for PathBuf {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&CStr) -> T,
    {
        self.as_os_str().with_nix_path(f)
    }
}
