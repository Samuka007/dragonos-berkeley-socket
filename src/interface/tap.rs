use std::{ops::DerefMut, sync::Arc, time::Instant};

use smoltcp::{
    iface::{Config, Interface},
    wire::HardwareAddress,
};
use spin::Mutex;

use crate::driver::tap::TapDevice;

use super::{Iface, IfaceCommon};

#[derive(Debug)]
pub struct TapIface {
    pub inner: Arc<Mutex<TapDevice>>,
    pub common: IfaceCommon,
}

// #[derive(Debug)]
// pub struct TapIface (Arc<Mutex<TapIfaceInner>>);

impl TapIface {
    pub fn new(inner: Arc<Mutex<TapDevice>>) -> Self {
        let mut iface_config = Config::new(HardwareAddress::Ethernet(inner.lock().mac()));
        iface_config.random_seed = std::random::random();

        let iface = Interface::new(
            iface_config,
            inner.lock().deref_mut(),
            Instant::now().into(),
        );
        let common = IfaceCommon::new(1, true, iface);
        TapIface { inner, common }
    }
}

unsafe impl Send for TapIface {}
unsafe impl Sync for TapIface {}

impl Iface for TapIface {
    fn common(&self) -> &IfaceCommon {
        &self.common
    }

    fn mac(&self) -> smoltcp::wire::EthernetAddress {
        self.inner.lock().mac()
    }

    // fn iface_name(&self) -> String {
    //     "tap0".to_string()
    // }

    // fn nic_id(&self) -> usize {
    //     self.inner.nic_id()
    // }

    fn poll(&self) {
        let mut guard = self.inner.lock();
        let reference = guard.deref_mut();
        self.common.poll(reference);
    }
}
