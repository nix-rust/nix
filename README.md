# Rust bindings to *nix APIs

[![Build Status](https://travis-ci.org/nix-rust/nix.svg?branch=master)](https://travis-ci.org/nix-rust/nix)
[![crates.io](http://meritbadge.herokuapp.com/nix)](https://crates.io/crates/nix)

[Documentation (Releases)](https://docs.rs/nix/)

[Documentation (Development)](https://nix-rust.github.io/nix/nix/index.html)

Nix seeks to provide friendly bindings to various *nix platform APIs (Linux, Darwin,
...). The goal is to not provide a 100% unified interface, but to unify
what can be while still providing platform specific APIs.

For many system APIs, Nix provides a safe alternative to the unsafe APIs
exposed by the [libc crate](https://github.com/rust-lang/libc).  This is done by
wrapping the libc functionality with types/abstractions that enforce legal/safe
usage.


As an example of what Nix provides, examine the differences between what is
exposed by libc and nix for the
[gethostname](http://man7.org/linux/man-pages/man2/gethostname.2.html) system
call:

```rust,ignore
// libc api (unsafe, requires handling return code/errno)
pub unsafe extern fn gethostname(name: *mut c_char, len: size_t) -> c_int;

// nix api (returns a nix::Result)
pub fn gethostname(name: &mut [u8]) -> Result<()>;
```

## Supported Platforms

nix target support consists of two tiers. While nix attempts to support all
platforms supported by [libc](https://github.com/rust-lang/libc), only some
platforms are actively supported due to either technical or manpower
limitations. Support for platforms is split into two tiers:

  * Tier 1 - Builds and tests for this target are run in CI. Failures of either
             block the inclusion of new code.
  * Tier 2 - Builds for this target are run in CI. Failures during the build
             blocks the inclusion of new code. Tests may be run, but failures
             in tests don't block the inclusion of new code.

The following targets are all supported by nix on Rust 1.13.0 or newer:

Tier 1:
  * i686-unknown-linux-gnu
  * x86_64-unknown-linux-gnu
  * i686-apple-darwin
  * x86_64-apple-darwin
  * aarch64-unknown-linux-gnu
  * armv7-unknown-linux-gnueabihf
  * arm-unknown-linux-gnueabi
  * x86_64-unknown-freebsd
  * powerpc-unknown-linux-gnu
  * mips-unknown-linux-gnu
  * mipsel-unknown-linux-gnu

Tier 2:
  * i686-unknown-freebsd
  * x86_64-unknown-netbsd
  * i686-unknown-linux-musl
  * x86_64-unknown-linux-musl

## Usage

To use `nix`, first add this to your `Cargo.toml`:

```toml
[dependencies]
nix = "0.8.0"
```

Then, add this to your crate root:

```rust,ignore
extern crate nix;
```
## Contributing

Contributions are very welcome.  Please See [CONTRIBUTING](CONTRIBUTING.md) for
additional details.

Feel free to join us in [the nix-rust/nix](https://gitter.im/nix-rust/nix) channel on Gitter to
discuss `nix` development.

## License

Nix is licensed under the MIT license.  See [LICENSE](LICENSE) for more details.
