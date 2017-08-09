/// The `libc_bitflags!` macro helps with a common use case of defining bitflags with values from
/// the libc crate. It is used the same way as the `bitflags!` macro, except that only the name of
/// the flag value has to be given.
///
/// The `libc` crate must be in scope with the name `libc`.
///
/// # Example
/// ```
/// libc_bitflags!{
///     pub struct ProtFlags: libc::c_int {
///         PROT_NONE;
///         PROT_READ;
///         PROT_WRITE;
///         PROT_EXEC;
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSDOWN;
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSUP;
///     }
/// }
/// ```
///
/// Example with casting, due to a mistake in libc. In this example, the
/// various flags have different types, so we cast the broken ones to the right
/// type.
///
/// ```
/// libc_bitflags!{
///     pub struct SaFlags: libc::c_ulong {
///         SA_NOCLDSTOP as libc::c_ulong;
///         SA_NOCLDWAIT;
///         SA_NODEFER as libc::c_ulong;
///         SA_ONSTACK;
///         SA_RESETHAND as libc::c_ulong;
///         SA_RESTART as libc::c_ulong;
///         SA_SIGINFO;
///     }
/// }
/// ```
macro_rules! libc_bitflags {
    // (non-pub) Exit rule.
    (@call_bitflags
        {
            name: $BitFlags:ident,
            type: $T:ty,
            attrs: [$($attrs:tt)*],
            flags: [$($flags:tt)*],
        }
    ) => {
        bitflags! {
            $($attrs)*
            struct $BitFlags: $T {
                $($flags)*
            }
        }
    };

    // (pub) Exit rule.
    (@call_bitflags
        {
            pub,
            name: $BitFlags:ident,
            type: $T:ty,
            attrs: [$($attrs:tt)*],
            flags: [$($flags:tt)*],
        }
    ) => {
        bitflags! {
            $($attrs)*
            pub struct $BitFlags: $T {
                $($flags)*
            }
        }
    };

    // (non-pub) Done accumulating.
    (@accumulate_flags
        {
            name: $BitFlags:ident,
            type: $T:ty,
            attrs: $attrs:tt,
        },
        $flags:tt;
    ) => {
        libc_bitflags! {
            @call_bitflags
            {
                name: $BitFlags,
                type: $T,
                attrs: $attrs,
                flags: $flags,
            }
        }
    };

    // (pub) Done accumulating.
    (@accumulate_flags
        {
            pub,
            name: $BitFlags:ident,
            type: $T:ty,
            attrs: $attrs:tt,
        },
        $flags:tt;
    ) => {
        libc_bitflags! {
            @call_bitflags
            {
                pub,
                name: $BitFlags,
                type: $T,
                attrs: $attrs,
                flags: $flags,
            }
        }
    };

    // Munch an attr.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        #[$attr:meta] $($tail:tt)*
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                #[$attr]
            ];
            $($tail)*
        }
    };

    // Munch last ident if not followed by a semicolon.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        $flag:ident
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                const $flag = libc::$flag;
            ];
        }
    };

    // Munch last ident and cast it to the given type.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        $flag:ident as $ty:ty
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                const $flag = libc::$flag as $ty;
            ];
        }
    };

    // Munch an ident; covers terminating semicolon case.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        $flag:ident; $($tail:tt)*
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                const $flag = libc::$flag;
            ];
            $($tail)*
        }
    };

    // Munch an ident and cast it to the given type; covers terminating semicolon
    // case.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        $flag:ident as $ty:ty; $($tail:tt)*
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                const $flag = libc::$flag as $ty;
            ];
            $($tail)*
        }
    };

    // (non-pub) Entry rule.
    (
        $(#[$attr:meta])*
        struct $BitFlags:ident: $T:ty {
            $($vals:tt)*
        }
    ) => {
        libc_bitflags! {
            @accumulate_flags
            {
                name: $BitFlags,
                type: $T,
                attrs: [$(#[$attr])*],
            },
            [];
            $($vals)*
        }
    };

    // (pub) Entry rule.
    (
        $(#[$attr:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $($vals:tt)*
        }
    ) => {
        libc_bitflags! {
            @accumulate_flags
            {
                pub,
                name: $BitFlags,
                type: $T,
                attrs: [$(#[$attr])*],
            },
            [];
            $($vals)*
        }
    };
}

/// The `libc_enum!` macro helps with a common use case of defining an enum exclusively using
/// values from the `libc` crate. This macro supports both `pub` and private `enum`s.
///
/// The `libc` crate must be in scope with the name `libc`.
///
/// # Example
/// ```
/// libc_enum!{
///     pub enum ProtFlags {
///         PROT_NONE,
///         PROT_READ,
///         PROT_WRITE,
///         PROT_EXEC,
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSDOWN,
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSUP,
///     }
/// }
/// ```
macro_rules! libc_enum {
    // (non-pub) Exit rule.
    (@make_enum
        {
            name: $BitFlags:ident,
            attrs: [$($attrs:tt)*],
            entries: [$($entries:tt)*],
        }
    ) => {
        $($attrs)*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        enum $BitFlags {
            $($entries)*
        }
    };

    // (pub) Exit rule.
    (@make_enum
        {
            pub,
            name: $BitFlags:ident,
            attrs: [$($attrs:tt)*],
            entries: [$($entries:tt)*],
        }
    ) => {
        $($attrs)*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $BitFlags {
            $($entries)*
        }
    };

    // (non-pub) Done accumulating.
    (@accumulate_entries
        {
            name: $BitFlags:ident,
            attrs: $attrs:tt,
        },
        $entries:tt;
    ) => {
        libc_enum! {
            @make_enum
            {
                name: $BitFlags,
                attrs: $attrs,
                entries: $entries,
            }
        }
    };

    // (pub) Done accumulating.
    (@accumulate_entries
        {
            pub,
            name: $BitFlags:ident,
            attrs: $attrs:tt,
        },
        $entries:tt;
    ) => {
        libc_enum! {
            @make_enum
            {
                pub,
                name: $BitFlags,
                attrs: $attrs,
                entries: $entries,
            }
        }
    };

    // Munch an attr.
    (@accumulate_entries
        $prefix:tt,
        [$($entries:tt)*];
        #[$attr:meta] $($tail:tt)*
    ) => {
        libc_enum! {
            @accumulate_entries
            $prefix,
            [
                $($entries)*
                #[$attr]
            ];
            $($tail)*
        }
    };

    // Munch last ident if not followed by a comma.
    (@accumulate_entries
        $prefix:tt,
        [$($entries:tt)*];
        $entry:ident
    ) => {
        libc_enum! {
            @accumulate_entries
            $prefix,
            [
                $($entries)*
                $entry = libc::$entry,
            ];
        }
    };

    // Munch an ident; covers terminating comma case.
    (@accumulate_entries
        $prefix:tt,
        [$($entries:tt)*];
        $entry:ident, $($tail:tt)*
    ) => {
        libc_enum! {
            @accumulate_entries
            $prefix,
            [
                $($entries)*
                $entry = libc::$entry,
            ];
            $($tail)*
        }
    };

    // (non-pub) Entry rule.
    (
        $(#[$attr:meta])*
        enum $BitFlags:ident {
            $($vals:tt)*
        }
    ) => {
        libc_enum! {
            @accumulate_entries
            {
                name: $BitFlags,
                attrs: [$(#[$attr])*],
            },
            [];
            $($vals)*
        }
    };

    // (pub) Entry rule.
    (
        $(#[$attr:meta])*
        pub enum $BitFlags:ident {
            $($vals:tt)*
        }
    ) => {
        libc_enum! {
            @accumulate_entries
            {
                pub,
                name: $BitFlags,
                attrs: [$(#[$attr])*],
            },
            [];
            $($vals)*
        }
    };
}

/// A Rust version of the familiar C `offset_of` macro.  It returns the byte
/// offset of `field` within struct `ty`
macro_rules! offset_of {
    ($ty:ty, $field:ident) => {
        &(*(0 as *const $ty)).$field as *const _ as usize
    }
}
