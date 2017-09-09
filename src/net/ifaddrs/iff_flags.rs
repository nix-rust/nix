use libc;
use libc::c_uint;

bitflags! {
    pub struct IffFlags: c_uint {
        /// Interface is running.
        const IFF_UP = libc::IFF_UP as c_uint;
        /// Valid broadcast address set.
        const IFF_BROADCAST = libc::IFF_BROADCAST as c_uint;
        /// Internal debugging flag.
        const IFF_DEBUG = libc::IFF_DEBUG as c_uint;
        /// Interface is a loopback interface.
        const IFF_LOOPBACK = libc::IFF_LOOPBACK as c_uint;
        /// Interface is a point-to-point link.
        const IFF_POINTOPOINT = libc::IFF_POINTOPOINT as c_uint;
        /// Resources allocated.
        const IFF_NOTRAILERS = libc::IFF_NOTRAILERS as c_uint;
        /// No arp protocol, L2 destination address not set.
        const IFF_RUNNING = libc::IFF_RUNNING as c_uint;
        /// Interface is in promiscuous mode.
        const IFF_NOARP = libc::IFF_NOARP as c_uint;
        /// Avoid use of trailers.
        const IFF_PROMISC = libc::IFF_PROMISC as c_uint;
        /// Receive all multicast packets.
        const IFF_ALLMULTI = libc::IFF_ALLMULTI as c_uint;
        /// Master of a load balancing bundle.
        const IFF_MASTER = libc::IFF_MASTER as c_uint;
        /// Slave of a load balancing bundle.
        const IFF_SLAVE = libc::IFF_SLAVE as c_uint;
        /// Supports multicast
        const IFF_MULTICAST = libc::IFF_MULTICAST as c_uint;
        /// Is able to select media type via ifmap.
        const IFF_PORTSEL = libc::IFF_PORTSEL as c_uint;
        /// Auto media selection active.
        const IFF_AUTOMEDIA = libc::IFF_AUTOMEDIA as c_uint;
        /// The addresses are lost when the interface goes down.
        const IFF_DYNAMIC = libc::IFF_DYNAMIC as c_uint;

        // These flags are available on modern Linuxes
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Driver signals L1 up (since Linux 2.6.17)
        const IFF_LOWER_UP = 1<<16;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Driver signals dormant (since Linux 2.6.17)
        const IFF_DORMANT = 1<<17;
        #[cfg(any(target_os = "linux", target_os = "android"))]
        /// Echo sent packets (since Linux 2.6.25)
        const IFF_ECHO = 1<<18;
    }
}

