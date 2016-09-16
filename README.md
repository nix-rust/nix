# Rust bindings to *nix APIs

[![Build Status](https://travis-ci.org/nix-rust/nix.svg?branch=master)](https://travis-ci.org/nix-rust/nix)
[![crates.io](http://meritbadge.herokuapp.com/nix)](https://crates.io/crates/nix)

[Documentation](https://nix-rust.github.io/nix/nix/index.html)

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

## Usage

To use `nix`, first add this to your `Cargo.toml`:

```toml
[dependencies]
nix = "0.7.0"
```

Then, add this to your crate root:

```rust,ignore
extern crate nix;
```
## Contributing

Contributions are very welcome.  Please See [CONTRIBUTING](CONTRIBUTING.md) for
additional details.

## License

Nix is licensed under the MIT license.  See [LICENSE](LICENSE) for more details.
