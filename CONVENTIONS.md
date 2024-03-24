# Conventions

In order to achieve our goal of wrapping [libc][libc] code in idiomatic rust
constructs with minimal performance overhead, we follow the following
conventions.

Note that, thus far, not all the code follows these conventions and not all
conventions we try to follow have been documented here. If you find an instance
of either, feel free to remedy the flaw by opening a pull request with
appropriate changes or additions.

## Change Log

We follow the conventions laid out in [Keep A CHANGELOG][kacl].

[kacl]: https://github.com/olivierlacan/keep-a-changelog/tree/18adb5f5be7a898d046f6a4acb93e39dcf40c4ad

## libc constants, functions and structs

We do not define ffi functions or their associated constants and types ourselves,
but use or reexport them from the [libc crate][libc], if your PR uses something 
that does not exist in the libc crate, you should add it to libc first. Once 
your libc PR gets merged, you can adjust our `libc` dependency to include that 
libc change. Use a git dependency if necessary.

```toml
libc = { git = "https://github.com/rust-lang/libc", rev = "the commit includes your libc PR", ... }
```

We use the functions exported from [libc][libc] instead of writing our own
`extern` declarations.

We use the `struct` definitions from [libc][libc] internally instead of writing
our own. If we want to add methods to a libc type, we use the newtype pattern.
For example,

```rust
pub struct SigSet(libc::sigset_t);

impl SigSet {
    ...
}
```

When creating newtypes, we use Rust's `CamelCase` type naming convention.

## cfg gates

When creating operating-system-specific functionality, we gate it by
`#[cfg(target_os = ...)]`. If more than one operating system is affected, we
prefer to use the cfg aliases defined in build.rs, like `#[cfg(bsd)]`.

## Bitflags

Many C functions have flags parameters that are combined from constants using
bitwise operations. We represent the types of these parameters by types defined
using our `libc_bitflags!` macro, which is a convenience wrapper around the
`bitflags!` macro from the [bitflags crate][bitflags] that brings in the
constant value from `libc`.

We name the type for a set of constants whose element's names start with `FOO_`
`FooFlags`.

For example,

```rust
libc_bitflags!{
    pub struct ProtFlags: libc::c_int {
        PROT_NONE;
        PROT_READ;
        PROT_WRITE;
        PROT_EXEC;
        #[cfg(linux_android)]
        PROT_GROWSDOWN;
        #[cfg(linux_android)]
        PROT_GROWSUP;
    }
}
```


## Enumerations

We represent sets of constants that are intended as mutually exclusive arguments
to parameters of functions by [enumerations][enum].


## Structures Initialized by libc Functions

Whenever we need to use a [libc][libc] function to properly initialize a
variable and said function allows us to use uninitialized memory, we use
[`std::mem::MaybeUninit`][std_MaybeUninit] when defining the variable. This
allows us to avoid the overhead incurred by zeroing or otherwise initializing
the variable.

[bitflags]: https://crates.io/crates/bitflags/
[enum]: https://doc.rust-lang.org/reference.html#enumerations
[libc]: https://crates.io/crates/libc/
[std_MaybeUninit]: https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html

## Pointer type casting

We prefer [`cast()`], [`cast_mut()`] and [`cast_const()`] to cast pointer types
over the `as` keyword because it is much more difficult to accidentally change
type or mutability that way.

[`cast()`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.cast
[`cast_mut()`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.cast_mut
[`cast_const()`]: https://doc.rust-lang.org/std/primitive.pointer.html#method.cast_const

## Remove/deprecate an interface

In Nix, if we want to remove something, we don't do it immediately, instead, we
deprecate it for at least one release before removing it.

To deprecate an interface, put the following attribute on the top of it:

```
#[deprecated(since = "<Version>", note = "<Note to our user>")]
```

`<Version>` is the version where this interface will be deprecated, in most 
cases, it will be the version of the next release. And a user-friendly note 
should be added. Normally, there should be a new interface that will replace
the old one, so a note should be something like: "`<New Interface>` should be 
used instead".

## Where to put a test

If you want to add a test for a feature that is in `xxx.rs`, then the test should
be put in the corresponding `test_xxx.rs` file unless you cannot do this, e.g.,
the test involves private stuff and thus cannot be added outside of Nix, then
it is allowed to leave the test in `xxx.rs`.
