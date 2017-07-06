# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

### Added
- Added `sys::signal::SigAction::{ flags, mask, handler}`
  ([#611](https://github.com/nix-rust/nix/pull/609)
- Added `nix::sys::pthread::pthread_self`
  ([#591](https://github.com/nix-rust/nix/pull/591)
- Added `AioCb::from_boxed_slice`
  ([#582](https://github.com/nix-rust/nix/pull/582)
- Added `nix::unistd::{openat, fstatat, readlink, readlinkat}`
  ([#551](https://github.com/nix-rust/nix/pull/551))
- Added `nix::pty::{grantpt, posix_openpt, ptsname/ptsname_r, unlockpt}`
  ([#556](https://github.com/nix-rust/nix/pull/556)
- Added `nix::ptr::openpty`
  ([#456](https://github.com/nix-rust/nix/pull/456))
- Added `nix::ptrace::{ptrace_get_data, ptrace_getsiginfo, ptrace_setsiginfo
  and nix::Error::UnsupportedOperation}`
  ([#614](https://github.com/nix-rust/nix/pull/614))

### Changed
- Changed ioctl! write to take argument by value instead as pointer.
  If you need a pointer as argument, use ioctl! write buf.
  ([#626](https://github.com/nix-rust/nix/pull/626))
- Marked `sys::mman::{ mmap, munmap, madvise, munlock, msync }` as unsafe.
  ([#559](https://github.com/nix-rust/nix/pull/559))
- Minimum supported Rust version is now 1.13
- Removed `revents` argument from `PollFd::new()` as it's an output argument and
  will be overwritten regardless of value.
  ([#542](https://github.com/nix-rust/nix/pull/542))
- Changed type signature of `sys::select::FdSet::contains` to make `self`
  immutable ([#564](https://github.com/nix-rust/nix/pull/564))
- Changed type of `sched::sched_setaffinity`'s `pid` argument to `pid_t`
- Introduced wrapper types for gid_t, pid_t, and uid_t as Gid, Pid, and Uid
  respectively. Various functions have been changed to use these new types as
  arguments. ([#629](https://github.com/nix-rust/nix/pull/629))
- Promoted all Android targets to Tier 2 support

### Removed
- Removed io::Error from nix::Error and conversion from nix::Error to Errno
  ([#614](https://github.com/nix-rust/nix/pull/614))

### Fixed
- Fixed multiple issues compiling under different archetectures and OSes.
  Now compiles on Linux/MIPS ([#538](https://github.com/nix-rust/nix/pull/538)),
  `Linux/PPC` ([#553](https://github.com/nix-rust/nix/pull/553)),
  `MacOS/x86_64,i686` ([#553](https://github.com/nix-rust/nix/pull/553)),
  `NetBSD/x64_64` ([#538](https://github.com/nix-rust/nix/pull/538)),
  `FreeBSD/x86_64,i686` ([#536](https://github.com/nix-rust/nix/pull/536)), and
  `Android` ([#631](https://github.com/nix-rust/nix/pull/631)).
- `bind` and `errno_location` now work correctly on `Android`
  ([#631](https://github.com/nix-rust/nix/pull/631))

## [0.8.1] 2017-04-16

### Fixed
- Fixed build on FreeBSD. (Cherry-picked
  [a859ee3c](https://github.com/nix-rust/nix/commit/a859ee3c9396dfdb118fcc2c8ecc697e2d303467))

## [0.8.0] 2017-03-02

### Added
- Added `::nix::sys::termios::BaudRate` enum to provide portable baudrate
  values. ([#518](https://github.com/nix-rust/nix/pull/518))
- Added a new `WaitStatus::PtraceEvent` to support ptrace events on Linux
  and Android ([([#438](https://github.com/nix-rust/nix/pull/438))
- Added support for POSIX AIO
  ([#483](https://github.com/nix-rust/nix/pull/483))
  ([#506](https://github.com/nix-rust/nix/pull/506))
- Added support for XNU system control sockets
  ([#478](https://github.com/nix-rust/nix/pull/478))
- Added support for `ioctl` calls on BSD platforms
  ([#478](https://github.com/nix-rust/nix/pull/478))
- Added struct `TimeSpec`
  ([#475](https://github.com/nix-rust/nix/pull/475))
  ([#483](https://github.com/nix-rust/nix/pull/483))
- Added complete definitions for all kqueue-related constants on all supported
  OSes
  ([#415](https://github.com/nix-rust/nix/pull/415))
- Added function `epoll_create1` and bitflags `EpollCreateFlags` in
  `::nix::sys::epoll` in order to support `::libc::epoll_create1`.
  ([#410](https://github.com/nix-rust/nix/pull/410))
- Added `setresuid` and `setresgid` for Linux in `::nix::unistd`
  ([#448](https://github.com/nix-rust/nix/pull/448))
- Added `getpgid` in `::nix::unistd`
  ([#433](https://github.com/nix-rust/nix/pull/433))
- Added `tcgetpgrp` and `tcsetpgrp` in `::nix::unistd`
  ([#451](https://github.com/nix-rust/nix/pull/451))
- Added `CLONE_NEWCGROUP` in `::nix::sched`
  ([#457](https://github.com/nix-rust/nix/pull/457))
- Added `getpgrp` in `::nix::unistd`
  ([#491](https://github.com/nix-rust/nix/pull/491))
- Added `fchdir` in `::nix::unistd`
  ([#497](https://github.com/nix-rust/nix/pull/497))
- Added `major` and `minor` in `::nix::sys::stat` for decomposing `dev_t`
  ([#508](https://github.com/nix-rust/nix/pull/508))
- Fixed the style of many bitflags and use `libc` in more places.
  ([#503](https://github.com/nix-rust/nix/pull/503))
- Added `ppoll` in `::nix::poll`
  ([#520](https://github.com/nix-rust/nix/pull/520))
- Added support for getting and setting pipe size with fcntl(2) on Linux
  ([#540](https://github.com/nix-rust/nix/pull/540)

### Changed
- `::nix::sys::termios::{cfgetispeed, cfsetispeed, cfgetospeed, cfsetospeed}`
  switched  to use `BaudRate` enum from `speed_t`.
  ([#518](https://github.com/nix-rust/nix/pull/518))
- `epoll_ctl` now could accept None as argument `event`
  when op is `EpollOp::EpollCtlDel`.
  ([#480](https://github.com/nix-rust/nix/pull/480))
- Removed the `bad` keyword from the `ioctl!` macro
  ([#478](https://github.com/nix-rust/nix/pull/478))
- Changed `TimeVal` into an opaque Newtype
  ([#475](https://github.com/nix-rust/nix/pull/475))
- `kill`'s signature, defined in `::nix::sys::signal`, changed, so that the
  signal parameter has type `T: Into<Option<Signal>>`. `None` as an argument
  for that parameter will result in a 0 passed to libc's `kill`, while a
  `Some`-argument will result in the previous behavior for the contained
  `Signal`.
  ([#445](https://github.com/nix-rust/nix/pull/445))
- The minimum supported version of rustc is now 1.7.0.
  ([#444](https://github.com/nix-rust/nix/pull/444))
- Changed `KEvent` to an opaque structure that may only be modified by its
  constructor and the `ev_set` method.
  ([#415](https://github.com/nix-rust/nix/pull/415))
  ([#442](https://github.com/nix-rust/nix/pull/442))
  ([#463](https://github.com/nix-rust/nix/pull/463))
- `pipe2` now calls `libc::pipe2` where available. Previously it was emulated
  using `pipe`, which meant that setting `O_CLOEXEC` was not atomic.
  ([#427](https://github.com/nix-rust/nix/pull/427))
- Renamed `EpollEventKind` to `EpollFlags` in `::nix::sys::epoll` in order for
  it to conform with our conventions.
  ([#410](https://github.com/nix-rust/nix/pull/410))
- `EpollEvent` in `::nix::sys::epoll` is now an opaque proxy for
  `::libc::epoll_event`. The formerly public field `events` is now be read-only
  accessible with the new method `events()` of `EpollEvent`. Instances of
  `EpollEvent` can be constructed using the new method `new()` of EpollEvent.
  ([#410](https://github.com/nix-rust/nix/pull/410))
- `SigFlags` in `::nix::sys::signal` has be renamed to `SigmaskHow` and its type
  has changed from `bitflags` to `enum` in order to conform to our conventions.
  ([#460](https://github.com/nix-rust/nix/pull/460))
- `sethostname` now takes a `&str` instead of a `&[u8]` as this provides an API
  that makes more sense in normal, correct usage of the API.
- `gethostname` previously did not expose the actual length of the hostname
  written from the underlying system call at all.  This has been updated to
  return a `&CStr` within the provided buffer that is always properly
  NUL-terminated (this is not guaranteed by the call with all platforms/libc
  implementations).
- Exposed all fcntl(2) operations at the module level, so they can be
  imported direclty instead of via `FcntlArg` enum.
  ([#541](https://github.com/nix-rust/nix/pull/541))

### Fixed
- Fixed multiple issues with Unix domain sockets on non-Linux OSes
  ([#474](https://github.com/nix-rust/nix/pull/415))
- Fixed using kqueue with `EVFILT_USER` on FreeBSD
  ([#415](https://github.com/nix-rust/nix/pull/415))
- Fixed the build on FreeBSD, and fixed the getsockopt, sendmsg, and recvmsg
  functions on that same OS.
  ([#397](https://github.com/nix-rust/nix/pull/397))
- Fixed an off-by-one bug in `UnixAddr::new_abstract` in `::nix::sys::socket`.
  ([#429](https://github.com/nix-rust/nix/pull/429))
- Fixed clone passing a potentially unaligned stack.
  ([#490](https://github.com/nix-rust/nix/pull/490))
- Fixed mkdev not creating a `dev_t` the same way as libc.
  ([#508](https://github.com/nix-rust/nix/pull/508))

## [0.7.0] 2016-09-09

### Added
- Added `lseek` and `lseek64` in `::nix::unistd`
  ([#377](https://github.com/nix-rust/nix/pull/377))
- Added `mkdir` and `getcwd` in `::nix::unistd`
  ([#416](https://github.com/nix-rust/nix/pull/416))
- Added accessors `sigmask_mut` and `sigmask` to `UContext` in
  `::nix::ucontext`.
  ([#370](https://github.com/nix-rust/nix/pull/370))
- Added `WUNTRACED` to `WaitPidFlag` in `::nix::sys::wait` for non-_linux_
  targets.
  ([#379](https://github.com/nix-rust/nix/pull/379))
- Added new module `::nix::sys::reboot` with enumeration `RebootMode` and
  functions `reboot` and `set_cad_enabled`. Currently for _linux_ only.
  ([#386](https://github.com/nix-rust/nix/pull/386))
- `FdSet` in `::nix::sys::select` now also implements `Clone`.
  ([#405](https://github.com/nix-rust/nix/pull/405))
- Added `F_FULLFSYNC` to `FcntlArg` in `::nix::fcntl` for _apple_ targets.
  ([#407](https://github.com/nix-rust/nix/pull/407))
- Added `CpuSet::unset` in `::nix::sched`.
  ([#402](https://github.com/nix-rust/nix/pull/402))
- Added constructor method `new()` to `PollFd` in `::nix::poll`, in order to
  allow creation of objects, after removing public access to members.
  ([#399](https://github.com/nix-rust/nix/pull/399))
- Added method `revents()` to `PollFd` in `::nix::poll`, in order to provide
  read access to formerly public member `revents`.
  ([#399](https://github.com/nix-rust/nix/pull/399))
- Added `MSG_CMSG_CLOEXEC` to `MsgFlags` in `::nix::sys::socket` for _linux_ only.
  ([#422](https://github.com/nix-rust/nix/pull/422))

### Changed
- Replaced the reexported integer constants for signals by the enumeration
  `Signal` in `::nix::sys::signal`.
  ([#362](https://github.com/nix-rust/nix/pull/362))
- Renamed `EventFdFlag` to `EfdFlags` in `::nix::sys::eventfd`.
  ([#383](https://github.com/nix-rust/nix/pull/383))
- Changed the result types of `CpuSet::is_set` and `CpuSet::set` in
  `::nix::sched` to `Result<bool>` and `Result<()>`, respectively. They now
  return `EINVAL`, if an invalid argument for the `field` parameter is passed.
  ([#402](https://github.com/nix-rust/nix/pull/402))
- `MqAttr` in `::nix::mqueue` is now an opaque proxy for `::libc::mq_attr`,
  which has the same structure as the old `MqAttr`. The field `mq_flags` of
  `::libc::mq_attr` is readable using the new method `flags()` of `MqAttr`.
  `MqAttr` also no longer implements `Debug`.
  ([#392](https://github.com/nix-rust/nix/pull/392))
- The parameter `msq_prio` of `mq_receive` with type `u32` in `::nix::mqueue`
  was replaced by a parameter named `msg_prio` with type `&mut u32`, so that
  the message priority can be obtained by the caller.
  ([#392](https://github.com/nix-rust/nix/pull/392))
- The type alias `MQd` in `::nix::queue` was replaced by the type alias
  `libc::mqd_t`, both of which are aliases for the same type.
  ([#392](https://github.com/nix-rust/nix/pull/392))

### Removed
- Type alias `SigNum` from `::nix::sys::signal`.
  ([#362](https://github.com/nix-rust/nix/pull/362))
- Type alias `CpuMask` from `::nix::shed`.
  ([#402](https://github.com/nix-rust/nix/pull/402))
- Removed public fields from `PollFd` in `::nix::poll`. (See also added method
  `revents()`.
  ([#399](https://github.com/nix-rust/nix/pull/399))

### Fixed
- Fixed the build problem for NetBSD (Note, that we currently do not support
  it, so it might already be broken again).
  ([#389](https://github.com/nix-rust/nix/pull/389))
- Fixed the build on FreeBSD, and fixed the getsockopt, sendmsg, and recvmsg
  functions on that same OS.
  ([#397](https://github.com/nix-rust/nix/pull/397))

## [0.6.0] 2016-06-10

### Added
- Added `gettid` in `::nix::unistd` for _linux_ and _android_.
  ([#293](https://github.com/nix-rust/nix/pull/293))
- Some _mips_ support in `::nix::sched` and `::nix::sys::syscall`.
  ([#301](https://github.com/nix-rust/nix/pull/301))
- Added `SIGNALFD_SIGINFO_SIZE` in `::nix::sys::signalfd`.
  ([#309](https://github.com/nix-rust/nix/pull/309))
- Added new module `::nix::ucontext` with struct `UContext`. Currently for
  _linux_ only.
  ([#311](https://github.com/nix-rust/nix/pull/311))
- Added `EPOLLEXCLUSIVE` to `EpollEventKind` in `::nix::sys::epoll`.
  ([#330](https://github.com/nix-rust/nix/pull/330))
- Added `pause` to `::nix::unistd`.
  ([#336](https://github.com/nix-rust/nix/pull/336))
- Added `sleep` to `::nix::unistd`.
  ([#351](https://github.com/nix-rust/nix/pull/351))
- Added `S_IFDIR`, `S_IFLNK`, `S_IFMT` to `SFlag` in `::nix::sys::stat`.
  ([#359](https://github.com/nix-rust/nix/pull/359))
- Added `clear` and `extend` functions to `SigSet`'s implementation in
  `::nix::sys::signal`.
  ([#347](https://github.com/nix-rust/nix/pull/347))
- `sockaddr_storage_to_addr` in `::nix::sys::socket` now supports `sockaddr_nl`
  on _linux_ and _android_.
  ([#366](https://github.com/nix-rust/nix/pull/366))
- Added support for `SO_ORIGINAL_DST` in `::nix::sys::socket` on _linux_.
  ([#367](https://github.com/nix-rust/nix/pull/367))
- Added `SIGINFO` in `::nix::sys::signal` for the _macos_ target as well as
  `SIGPWR` and `SIGSTKFLT` in `::nix::sys::signal` for non-_macos_ targets.
  ([#361](https://github.com/nix-rust/nix/pull/361))

### Changed
- Changed the structure `IoVec` in `::nix::sys::uio`.
  ([#304](https://github.com/nix-rust/nix/pull/304))
- Replaced `CREATE_NEW_FD` by `SIGNALFD_NEW` in `::nix::sys::signalfd`.
  ([#309](https://github.com/nix-rust/nix/pull/309))
- Renamed `SaFlag` to `SaFlags` and `SigFlag` to `SigFlags` in
  `::nix::sys::signal`.
  ([#314](https://github.com/nix-rust/nix/pull/314))
- Renamed `Fork` to `ForkResult` and changed its fields in `::nix::unistd`.
  ([#332](https://github.com/nix-rust/nix/pull/332))
- Added the `signal` parameter to `clone`'s signature in `::nix::sched`.
  ([#344](https://github.com/nix-rust/nix/pull/344))
- `execv`, `execve`, and `execvp` now return `Result<Void>` instead of
  `Result<()>` in `::nix::unistd`.
  ([#357](https://github.com/nix-rust/nix/pull/357))

### Fixed
- Improved the conversion from `std::net::SocketAddr` to `InetAddr` in
  `::nix::sys::socket::addr`.
  ([#335](https://github.com/nix-rust/nix/pull/335))

## [0.5.0] 2016-03-01
