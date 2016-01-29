# Bitflags

We represent sets of constants whose values are mutually exclusive on a bit
level -- that is, for all flags `A` and `B` of the set with `A != B` holds
`A & B = 0` -- by types defined using the `bitflags!` macro from the
[bitflags crate](https://crates.io/crates/bitflags/).
Instead of providing the concrete values ourselves, we prefer taking the
constants  defined in [libc crate](https://crates.io/crates/libc/).
