# Bitflags

We represent sets of constants that are intended to be combined using bitwise
operations as parameters to functions by types defined using the `bitflags!`
macro from the [bitflags crate](https://crates.io/crates/bitflags/).
Instead of providing the concrete values ourselves, we prefer taking the
constants  defined in [libc crate](https://crates.io/crates/libc/).
