//! Safe wrappers around functions found in POSIX <netdb.h> header
//! 
//! https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netdb.h.html

// The <netdb.h> header may define the in_port_t type and the in_addr_t type as described in <netinet/in.h>.
// Simple integer type aliases, so we rexport
pub use libc::{in_port_t, in_addr_t};
pub use libc::socklen_t;
