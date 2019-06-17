/// The `libc_bitflags!` macro helps with a common use case of defining a public bitflags type
/// with values from the libc crate. It is used the same way as the `bitflags!` macro, except
/// that only the name of the flag value has to be given.
///
/// # Example
/// ```
/// libc_bitflags!{
///     pub struct ProtFlags: libc::c_int {
///         PROT_NONE;
///         PROT_READ;
///         /// PROT_WRITE enables write protect
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
    (
        $(#[$outer:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Flag:ident $(as $cast:ty)*;
            )+
        }
    ) => {
        bitflags! {
            $(#[$outer])*
            pub struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = ::libc::$Flag $(as $cast)*;
                )+
            }
        }
    };
}

/// The `libc_enum!` macro helps with a common use case of defining an enum exclusively using
/// values from the `libc` crate. The type after the enum name specifies the type of the constants
/// in `libc`. The macro will generate impls of `From` and `TryFrom` to convert between numeric and
/// enum values.
///
/// `TryFrom` is only implemented for Rust >= 1.34.0, where the trait is stable. An equivalent
/// `try_from` inherent method is made available regardless of the Rust version. `TryInto` should
/// not be used as long as the MSRV for nix is less than 1.34.0.
/// 
///
/// Documentation for each variant must be provided before any cfg attributes.
///
/// # Example
/// ```
/// libc_enum! {
///     pub enum ProtFlags: c_int {
///         PROT_NONE,
///         PROT_READ,
///         PROT_WRITE,
///         PROT_EXEC,
///         /// Documentation before cfg attribute.
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSDOWN,
///         #[cfg(any(target_os = "linux", target_os = "android"))]
///         PROT_GROWSUP,
///     }
/// }
///
/// let flag: c_int = ProtFlags::PROT_NONE.into();
/// let flag: ProtFlags = ProtFlags::try_from(::libc::PROT_NONE).unwrap();
/// ```
macro_rules! libc_enum {
    // pub
    (
        $(#[$enum_attr:meta])*
        pub $(($($scope:tt)*))* enum $($def:tt)*
    ) => {
        libc_enum! {
            @(pub $(($($scope)*))*)
            $(#[$enum_attr])*
            enum $($def)*
        }
    };

    // non-pub
    (
        $(#[$enum_attr:meta])*
        enum $($def:tt)*
    ) => {
        libc_enum! {
            @()
            $(#[$enum_attr])*
            enum $($def)*
        }
    };

    (
        @($($vis:tt)*)
        $(#[$enum_attr:meta])*
        enum $enum:ident : $prim:ty {
            $(
                $(#[doc = $var_doc:tt])*
                $(#[cfg($var_cfg:meta)])*
                $entry:ident
            ),* $(,)*
        }
    ) => {
        $(#[$enum_attr])* 
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        $($vis)* enum $enum {
            $(
                $(#[doc = $var_doc])*
                $(#[cfg($var_cfg)])*
                $entry = ::libc::$entry as isize
            ),*
        }

        impl $enum {
            pub fn try_from(value: $prim) -> std::result::Result<$enum, ::Error> {
                match value {
                    $(
                        $(#[cfg($var_cfg)])*
                        ::libc::$entry => Ok($enum::$entry),
                    )*
                    // don't think this Error is the correct one
                    _ => Err(::Error::invalid_argument())
                }
            }
        }

        impl std::convert::From<$enum> for $prim {
            fn from(value: $enum) -> $prim {
                match value {
                    $(
                        $(#[cfg($var_cfg)])*
                        $enum::$entry => ::libc::$entry,
                    )*
                }
            }
        }

        #[cfg(try_from)]
        impl std::convert::TryFrom<$prim> for $enum {
            type Error = ::Error;

            fn try_from(value: $prim) -> std::result::Result<$enum, Self::Error> {
                $enum::try_from(value)
            }
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
