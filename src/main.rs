use std::sync::Arc;

use berkeley_socket::{
    driver::{irq::start_network_polling_thread, tap::TapDevice},
    interface::tap::TapIface,
    socket::inet::common::NET_DEVICES,
};
use spin::Mutex;

fn main() {
    let device = TapDevice::new("tap0", smoltcp::phy::Medium::Ethernet).unwrap();
    let iface = Arc::new(TapIface::new(Arc::new(Mutex::new(device))));
    NET_DEVICES.write().insert(0, iface);
    let _ = start_network_polling_thread();
    // TODO: add socket tests
}
