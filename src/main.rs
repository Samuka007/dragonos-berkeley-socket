use std::{net::Ipv4Addr, sync::Arc};

use berkeley_socket::{
    driver::{irq::start_network_polling_thread, tap::TapDevice}, interface::{tap::TapIface, Iface}, posix::SOCK, socket::{endpoint::Endpoint, inet::{common::NET_DEVICES, syscall::Inet}, Family}
};
use smoltcp::wire::{IpAddress, IpEndpoint, Ipv4Cidr, IpCidr};
use spin::Mutex;

fn main() {
    env_logger::init();
    let device = TapDevice::new("tap0", smoltcp::phy::Medium::Ethernet).unwrap();
    let iface_inner = TapIface::new(Arc::new(Mutex::new(device)));

    let ip_cidr = IpCidr::Ipv4(Ipv4Cidr::new(
        Ipv4Addr::new(192, 168, 213, 2),
        24
    ));

    let ip_cidr = vec![ip_cidr];

    iface_inner.update_ip_addrs(&ip_cidr).unwrap();

    let iface = Arc::new(iface_inner);

    NET_DEVICES.write().insert(0, iface);
    scopeguard::defer!({
        NET_DEVICES.write().clear();
    });
    let _ = start_network_polling_thread();

    let socket = Inet::socket(SOCK::Datagram, 0).unwrap();
    socket.bind(Endpoint::Ip(
        IpEndpoint::new(
            IpAddress::v4(192, 168, 213, 2), 
            1234,
        )
    )).unwrap();
    let mut buffer = [0u8; 1024];
    
    loop {
        let len = socket.read(&mut buffer).unwrap();
        log::info!("Received {} bytes: {}", len, String::from_utf8_lossy(&buffer[..len]));
    }
}
