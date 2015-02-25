
use {NixResult, NixError};
use super::addr::ToInAddr;
use super::consts;
use libc::in_addr;
use std::fmt;

#[repr(C)]
#[derive(Copy)]
pub struct ip_mreq {
    pub imr_multiaddr: in_addr,
    pub imr_interface: in_addr,
}

impl fmt::Debug for ip_mreq {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "ip_mreq {{ imr_multiaddr: {{ s_addr: 0x{:x} }}, imr_interface: {{ s_addr: 0x{:x} }} }}",
                    self.imr_multiaddr.s_addr, self.imr_interface.s_addr)
    }
}

impl ip_mreq {
    pub fn new<T: ToInAddr, U: ToInAddr>(group: T, interface: Option<U>) -> NixResult<ip_mreq> {
        let group = match group.to_in_addr() {
            Some(group) => group,
            None => return Err(NixError::invalid_argument()),
        };

        let interface = match interface {
            Some(interface) => {
                match interface.to_in_addr() {
                    Some(interface) => interface,
                    None => return Err(NixError::invalid_argument()),
                }
            }
            None => in_addr { s_addr: consts::INADDR_ANY },
        };

        Ok(ip_mreq {
            imr_multiaddr: group,
            imr_interface: interface,
        })
    }
}
