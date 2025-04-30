//! Print all interfaces and interface addresses on the system, in a format
//! similar to ifconfig(8).
#![cfg(feature = "net")]
#[cfg(any(bsd, linux_android, target_os = "illumos"))]
fn main() {
    use nix::ifaddrs::getifaddrs;
    use nix::sys::socket::{SockaddrLike, SockaddrStorage};

    let addrs = getifaddrs().unwrap();
    let mut ifname = None;
    for addr in addrs {
        if ifname.as_ref() != Some(&addr.interface_name) {
            if ifname.is_some() {
                println!();
            }
            ifname = Some(addr.interface_name.clone());
            println!(
                "{}: flags={:x}<{}>",
                addr.interface_name,
                addr.flags.bits(),
                addr.flags
            );
        }
        if let Some(dl) = addr.address.as_ref().unwrap().as_link_addr() {
            if dl.addr().is_none() {
                continue;
            }
        }
        let family = addr
            .address
            .as_ref()
            .and_then(SockaddrStorage::family)
            .map(|af| format!("{af:?}"))
            .unwrap_or("".to_owned());
        match (
            &addr.address,
            &addr.netmask,
            &addr.broadcast,
            &addr.destination,
        ) {
            (Some(a), Some(nm), Some(b), None) => {
                println!("\t{family} {a} netmask {nm} broadcast {b}")
            }
            (Some(a), Some(nm), None, None) => {
                println!("\t{family} {a} netmask {nm}")
            }
            (Some(a), None, None, None) => println!("\t{family} {a}"),
            (Some(a), None, None, Some(d)) => println!("\t{family} {a} -> {d}"),
            x => todo!("{x:?}"),
        }
    }
}

#[cfg(not(any(bsd, linux_android, target_os = "illumos")))]
fn main() {}
