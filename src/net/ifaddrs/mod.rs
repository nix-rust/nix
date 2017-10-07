//! `ifaddrs` provides a safe interface for the system's network interface data.
//!
//! The `InterfaceAddrs` struct provides access to the system's network
//! interface data. You can either iterate over it or consume it and convert
//! it into an `InterfaceMap` (a `HashMap<String, Vec<InterfaceAddr>>`) for
//! more convenient access by interface name.
//!
//! # Examples
//!
//! You can access the basic information of the system in a few lines.
//! The following program prints all the known addresses.
//!
//! ```
//! use nix::net::ifaddrs::InterfaceAddrs;
//!
//! let addrs = InterfaceAddrs::getifaddrs()
//!     .expect("Failed to enumerate network interfaces.");
//!
//! for addr in addrs {
//!     println!("{}: {:?}", addr.name, addr.address);
//! }
//! ```
//!
//! The `IffFlags` struct provides access to info about the
//! state of an interface. This program prints the addresses of only
//! interfaces which are up.
//!
//! ```
//! use nix::net::ifaddrs::{InterfaceAddrs, iff_flags};
//!
//! let addrs = InterfaceAddrs::getifaddrs()
//!     .expect("Failed to eunmerate network interfaces.");
//!
//! for addr in addrs {
//!     if addr.flags.contains(iff_flags::IFF_UP) {
//!         println!("{}: {:?}", addr.name, addr.address);
//!     }
//! }
//! ```
//!
//! You can convert the `InterfaceAddrs` struct into a `HashMap` easily.
//! `InterfaceMap` is an alias for `HashMap<String, Vec<InterfaceAddr>>` for
//! easier reference.
//!
//! ```
//! use nix::net::ifaddrs::{InterfaceAddrs, InterfaceAddr, InterfaceMap};
//! use std::collections::HashMap;
//!
//! let interfaces: InterfaceMap =
//!     InterfaceAddrs::getifaddrs()
//!     .expect("Failed to enumerate network interfaces.")
//!     .into(); // Convert to a hash map
//!
//! // Print all the addresses of the loopback interface
//! if let Some(addrs) = interfaces.get("lo") {
//!    println!("Loopback addresses:");
//!    for addr in addrs {
//!         println!("\t{:?}", addr);
//!    }
//! }
//!
//! ```
//!

use libc;
use std::ptr::null_mut;
use std::ffi::CStr;
use std::collections::HashMap;
use errno::{Errno, errno};
use Error;
use Result;

pub mod iff_flags;
use self::iff_flags::IffFlags;

mod sockaddr;
use self::sockaddr::{IfAddrValue, sockaddr_to_ifaddrvalue};

mod tests;

pub type InterfaceMap<'a> = HashMap<String, Vec<InterfaceAddr<'a>>>;

/// Represents a handle into the operating system's knowledge about network
/// interfaces present on the system. Allows the user to iterate over
/// interface configurations.
pub struct InterfaceAddrs<'a> {
    inner: *mut libc::ifaddrs,
    current: Option<&'a libc::ifaddrs>,
    do_free: bool,
}

impl<'a> InterfaceAddrs<'a> {
    /// Creates an `InterfaceAddrs` from a raw pointer, without calling into
    /// the `libc`.
    ///
    /// The destructor will not attempt to free memory on an InterfaceAddrs
    /// created in this way.
    ///
    /// # Unsafety
    /// The caller is responsible for making sure the given pointer is not
    /// in invalid memory.
    ///
    /// # Errors
    /// `Err(())` will be returned if `p` was void.
    pub unsafe fn from_raw(p: *mut libc::ifaddrs) -> ::std::result::Result<InterfaceAddrs<'a>, ()> {
        match p.as_ref() {
            Some(r) => Ok(Self {
                inner: p,
                current: Some(r),
                do_free: false,
            }),
            None => Err(()),
        }
    }

    /// Produce an `InterfaceAddrs` from the system's information.
    pub fn getifaddrs() -> Result<Self> {
        let mut p = null_mut();

        unsafe {
            libc::getifaddrs(&mut p);
        }

        // UNSAFETY: *mut -> &'static mut. This is known to be either in valid
        // memory or null based on the guarantees of getifaddrs()
        match unsafe { p.as_ref() } {
            Some(r) => Ok(Self {
                inner: p,
                current: Some(r),
                do_free: true,
            }),

            None => Err(Error::from(Errno::from_i32(errno()))),
        }
    }
}

impl<'a> From<InterfaceAddrs<'a>> for HashMap<String, Vec<InterfaceAddr<'a>>> {
    /// Collect an `InterfaceAddrs` into a `HashMap<String, InterfaceAddr>`.
    fn from(ia: InterfaceAddrs<'a>) -> HashMap<String, Vec<InterfaceAddr<'a>>> {
        let mut m = HashMap::new();
        for i in ia {
            if !m.contains_key(&i.name) {
                m.insert(i.name.clone(), Vec::new());
            }
            // Unwrap here because contains is checked above
            m.get_mut(&i.name).unwrap().push(i);
        }

        m
    }
}

impl<'a> Drop for InterfaceAddrs<'a> {
    fn drop(&mut self) {
        if self.do_free {
            // UNSAFETY: Calling libc FFI function which frees previously allocated
            // memory.
            unsafe {
                // Ask the libc to drop free the memory it allocated when
                // the struct was created.
                libc::freeifaddrs(self.inner as *mut libc::ifaddrs);
            }
        }
    }
}


/// Represents the configuration and state of a network interface.
/// Interfaces are uniquely identified by name, and each interface is likely
/// to be referred to multiple times, e.g. one for IPv4 and one for IPv6.
#[derive(Debug, Clone)]
pub struct InterfaceAddr<'a> {
    /// The name of the interface
    pub name: String,

    /// The address assigned to the interface for this protocol.
    /// A value of `None` means the libc reported a type of address that
    /// `std::net` doesn't understand.
    pub address: Option<IfAddrValue<'a>>,

    /// The netmasks assigned to the interface for this protocol.
    /// A value of `None` means the libc reported a type of address that
    /// `std::net` doesn't understand.
    pub netmask: Option<IfAddrValue<'a>>,

    /// The ifu assigned to the interface for this protocol.
    /// A value of `{Broadcast, Destination}Addr(None)` means the libc reported
    /// a type of address that `std::net` doesn't understand, while a value of
    /// `Neither` means that the interface has neither a valid broadcast address
    /// nor a point-to-point destination address.
    pub ifu: InterfaceIfu<'a>,

    /// Flags regarding the interface's behaviour and state
    pub flags: IffFlags,
}

/// Represents the ifu of an interface: either its broadcast address or
/// point-to-point destination address.
#[derive(Debug, Clone)]
pub enum InterfaceIfu<'a> {
    BroadcastAddr(Option<IfAddrValue<'a>>),
    DestinationAddr(Option<IfAddrValue<'a>>),
    Neither,
}


impl<'a> Iterator for InterfaceAddrs<'a> {
    type Item = InterfaceAddr<'a>;
    fn next(&mut self) -> Option<InterfaceAddr<'a>> {
        // If the current ifaddrs is None, there are no more ifaddrs to inspect
        if self.current.is_none() {
            return None;
        }

        // Workaround for the borrow checker being overzealous
        // (without ptr_temp, self.current would technically
        // "still be in use" when the loop ends, meaning we
        // couldn't advance to the next struct)
        let ptr_temp = self.current.clone();
        let p = ptr_temp.as_ref().unwrap();

        // Get a pointer to the interface's name
        let name_ptr = p.ifa_name;
        // Check that name_ptr isn't null.
        if name_ptr.is_null() {
            panic!("getifaddrs() gave an ifaddrs struct with a null ifa_name");
        }

        // UNSAFETY: Constructing CStr from pointer. If this pointer is
        // null it's a libc bug; it's checked above.
        let name = unsafe { CStr::from_ptr(name_ptr) }
            .to_string_lossy()
            .into_owned();

        // Interpret the flags field into a typed version of those flags
        let flags = IffFlags::from_bits_truncate(p.ifa_flags);

        // Get IfAddrValue representations of the address and netmask
        // UNSAFETY: sockaddr_to_ifaddrvalue requires valid pointer.
        let address = unsafe { sockaddr_to_ifaddrvalue(p.ifa_addr) };
        // UNSAFETY: sockaddr_to_ifaddrvalue requires valid pointer.
        let netmask = unsafe { sockaddr_to_ifaddrvalue(p.ifa_netmask) };

        // Figure out which ifu type is needed and create it
        let ifu = if flags.contains(iff_flags::IFF_POINTOPOINT) {
            // Point to point destination address
            // UNSAFETY: sockaddr_to_ifaddrvalue requires valid pointer.
            let ifu_addr = unsafe { sockaddr_to_ifaddrvalue(p.ifa_ifu) };
            InterfaceIfu::DestinationAddr(ifu_addr)
        } else if flags.contains(iff_flags::IFF_BROADCAST) {
            // Broadcast address
            // UNSAFETY: sockaddr_to_ifaddrvalue requires valid pointer.
            let ifu_addr = unsafe { sockaddr_to_ifaddrvalue(p.ifa_ifu) };
            InterfaceIfu::BroadcastAddr(ifu_addr)
        } else {
            InterfaceIfu::Neither
        };

        // Move along the list to the next ifaddrs struct
        // UNSAFETY: *mut -> Option<&'static mut>.
        // This is known to be in valid memory or null.
        self.current = unsafe { p.ifa_next.as_ref() };

        Some(InterfaceAddr {
            name: name,
            address: address,
            netmask: netmask,
            ifu: ifu,
            flags: flags,
        })
    }
}
