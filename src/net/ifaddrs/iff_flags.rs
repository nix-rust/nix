use libc;
use libc::c_uint;

libc_bitflags! {
    pub struct IffFlags: c_uint {
        /// Interface is running.
        IFF_UP as c_uint;
        /// Valid broadcast address set.
        IFF_BROADCAST as c_uint;
        /// Internal debugging flag.
        IFF_DEBUG as c_uint;
        /// Interface is a loopback interface.
        IFF_LOOPBACK as c_uint;
        /// Interface is a point-to-point link.
        IFF_POINTOPOINT as c_uint;
        /// Resources allocated.
        IFF_NOTRAILERS as c_uint;
        /// No arp protocol, L2 destination address not set.
        IFF_RUNNING as c_uint;
        /// Interface is in promiscuous mode.
        IFF_NOARP as c_uint;
        /// Avoid use of trailers.
        IFF_PROMISC as c_uint;
        /// Receive all multicast packets.
        IFF_ALLMULTI as c_uint;
        /// Master of a load balancing bundle.
        IFF_MASTER as c_uint;
        /// Slave of a load balancing bundle.
        IFF_SLAVE as c_uint;
        /// Supports multicast
        IFF_MULTICAST as c_uint;
        /// Is able to select media type via ifmap.
        IFF_PORTSEL as c_uint;
        /// Auto media selection active.
        IFF_AUTOMEDIA as c_uint;
        /// The addresses are lost when the interface goes down.
        IFF_DYNAMIC as c_uint;

        // These flags are available on modern Linuxes
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Driver signals L1 up (since Linux 2.6.17)
        IFF_LOWER_UP as c_uint;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Driver signals dormant (since Linux 2.6.17)
        IFF_DORMANT as c_uint;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Echo sent packets (since Linux 2.6.25)
        IFF_ECHO as c_uint;
    }
}
