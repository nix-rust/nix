#![cfg(test)]

use std::ptr::null_mut;
use std::mem::transmute;

use std::ffi::CString;

use libc;
use libc::{ifaddrs, sockaddr, sockaddr_in};
use libc::in_addr;

use super::iff_flags::*;
use super::InterfaceAddrs;
use super::InterfaceMap;

// Utility function to turn some bytes into a u32 in the right order because
// endianness is a pain
fn convert_v4_addr(b0: u8, b1: u8, b2: u8, b3: u8) -> in_addr {
    unsafe { in_addr { s_addr: transmute([b0, b1, b2, b3]) } }
}

// Utility function to turn some bytes into a sockaddr with the v4 addr type
fn sockaddr_for_addr(b0: u8, b1: u8, b2: u8, b3: u8) -> sockaddr {
    let sin = sockaddr_in {
        sin_family: libc::AF_INET as u16,
        sin_port: 0,
        sin_addr: convert_v4_addr(b0, b1, b2, b3),
        sin_zero: [0; 8],
    };

    unsafe { transmute(sin) }
}

#[test]
fn test_ifaddrs() {

    // Create an "external" interface
    let mut ext_address = sockaddr_for_addr(192, 168, 0, 1);
    let mut ext_netmask = sockaddr_for_addr(255, 255, 255, 0);
    let mut ext_brdcast = sockaddr_for_addr(192, 168, 0, 255);

    let mut test_ext_ipv4 = ifaddrs {
        ifa_next: null_mut(),
        ifa_name: CString::new("test_ext").unwrap().into_raw(),
        ifa_flags: (IFF_BROADCAST | IFF_UP | IFF_RUNNING).bits(),
        ifa_addr: &mut ext_address,
        ifa_netmask: &mut ext_netmask,
        ifa_ifu: &mut ext_brdcast,
        ifa_data: null_mut(),
    };


    // Create a "loopback" interface, and link it to the "external" one
    let mut lo_address = sockaddr_for_addr(127, 0, 0, 1);
    let mut lo_netmask = sockaddr_for_addr(255, 255, 255, 0);
    let mut lo_brdcast = sockaddr_for_addr(127, 0, 0, 255);

    let mut test_lo_ipv4 = ifaddrs {
        ifa_next: &mut test_ext_ipv4,
        ifa_name: CString::new("test_lo").unwrap().into_raw(),
        ifa_flags: (IFF_BROADCAST | IFF_LOOPBACK | IFF_UP | IFF_RUNNING).bits(),
        ifa_addr: &mut lo_address,
        ifa_netmask: &mut lo_netmask,
        ifa_ifu: &mut lo_brdcast,
        ifa_data: null_mut(),
    };

    let created = unsafe { InterfaceAddrs::from_raw(&mut test_lo_ipv4) }.unwrap();

    let hm: InterfaceMap = created.into();

    assert_eq!(hm.len(), 2, "Expected 2 interfaces, found {}.", hm.len());

    assert!(hm.contains_key("test_lo"));
    assert!(hm.contains_key("test_ext"));
}
