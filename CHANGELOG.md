# Change Log

All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

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
  ([#0](https://github.com/nix-rust/nix/pull/0))
- The parameter `msq_prio` of `mq_receive` with type `u32` in `::nix::mqueue`
  was replaced by a parameter named `msg_prio` with type `&mut u32`, so that
  the message priority can be obtained by the caller.
  ([#0](https://github.com/nix-rust/nix/pull/0))
- The type alias `MQd` in `::nix::queue` was replaced by the type alias
  `libc::mqd_t`, both of which are aliases for the same type.
  ([#0](https://github.com/nix-rust/nix/pull/0))

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
