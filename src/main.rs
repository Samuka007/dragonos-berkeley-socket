use std::sync::Arc;

use berkeley_socket::{
    driver::{irq::start_network_polling_thread, tap::TapDevice}, interface::tap::TapIface, posix::SOCK, socket::{endpoint::Endpoint, inet::{common::NET_DEVICES, syscall::Inet}, Family}
};
use smoltcp::wire::{IpAddress, IpEndpoint};
use spin::Mutex;

fn main() {
    let device = TapDevice::new("tap0", smoltcp::phy::Medium::Ethernet).unwrap();
    let iface = Arc::new(TapIface::new(Arc::new(Mutex::new(device))));
    NET_DEVICES.write().insert(0, iface);
    let _ = start_network_polling_thread();
    // TODO: add socket tests

    let socket = Inet::socket(SOCK::Datagram, 0).unwrap();
    socket.bind(Endpoint::Ip(
        IpEndpoint::new(
            IpAddress::v4(192, 168, 199, 1), 
            1234,
        )
    )).unwrap();
    let mut buffer = [0u8; 1024];
    socket.read(&mut buffer).unwrap();
    socket.write(&buffer).unwrap();
}
