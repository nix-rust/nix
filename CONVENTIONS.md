# Conventions

In order to achieve our goal of wrapping [libc][libc] code in idiomatic rust
constructs with minimal performance overhead, we follow the following
conventions.

Note that, thus far, not all the code follows these conventions and not all
conventions we try to follow have been documented here. If you find an instance
of either, feel free to remedy the flaw by opening a pull request with
appropriate changes or additions.


## libc constants, functions and structs

We do not define integer constants ourselves, but use or reexport them from the
[libc crate][libc].

We use the functions exported from [libc][libc] instead of writing our own
`extern` declarations.

We use the `struct` definitions from [libc][libc] internally instead of writing
our own.

## Bitflags

We represent sets of constants that are intended to be combined using bitwise
operations as parameters to functions by types defined using the `bitflags!`
macro from the [bitflags crate][bitflags].
We name the type for a set of constants whose element's names start with `FOO_`
`FooFlags`.


## Enumerations

We represent sets of constants that are intended as mutually exclusive arguments
to parameters of functions by [enumerations][enum].


## Structures Initialized by libc Functions

Whenever we need to use a [libc][libc] function to properly initialize a
variable and said function allows us to use uninitialized memory, we use
[`std::mem::uninitialized`][std_uninitialized] (or [`core::mem::uninitialized`][core_uninitialized])
when defining the variable. This allows us to avoid the overhead incurred by
zeroing or otherwise initializing the variable.

[bitflags]: https://crates.io/crates/bitflags/
[core_uninitialized]: https://doc.rust-lang.org/core/mem/fn.uninitialized.html
[enum]: https://doc.rust-lang.org/reference.html#enumerations
[libc]: https://crates.io/crates/libc/
[std_uninitialized]: https://doc.rust-lang.org/std/mem/fn.uninitialized.html
