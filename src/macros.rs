/// The `libc_bitflags!` macro helps with a common use case of defining bitflags with values from
/// the libc crate. It is used the same way as the `bitflags!` macro, except that only the name of
/// the flag value has to be given.
///
/// The `libc` crate must be in scope with the name `libc`.
///
/// # Example
/// ```
/// libc_bitflags!{
///     flags ProtFlags: libc::c_int {
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
            flags $BitFlags: $T {
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
            pub flags $BitFlags: $T {
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

    // Munch last ident if not followed by a comma.
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
                const $flag = libc::$flag,
            ];
        }
    };

    // Munch an ident; covers terminating comma case.
    (@accumulate_flags
        $prefix:tt,
        [$($flags:tt)*];
        $flag:ident, $($tail:tt)*
    ) => {
        libc_bitflags! {
            @accumulate_flags
            $prefix,
            [
                $($flags)*
                const $flag = libc::$flag,
            ];
            $($tail)*
        }
    };

    // (non-pub) Entry rule.
    (
        $(#[$attr:meta])*
        flags $BitFlags:ident: $T:ty {
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
        pub flags $BitFlags:ident: $T:ty {
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
