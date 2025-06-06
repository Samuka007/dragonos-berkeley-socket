use super::*;
use scopeguard::defer;
use smoltcp::{phy::Medium, wire::EthernetFrame};
use std::io;
use std::os::unix::io::{AsRawFd, RawFd};

#[derive(Debug)]
pub struct TapDesc {
    lower: libc::c_int,
    mtu: usize,
    mac: smoltcp::wire::EthernetAddress,
}

impl AsRawFd for TapDesc {
    fn as_raw_fd(&self) -> RawFd {
        self.lower
    }
}

impl TapDesc {
    pub fn new(name: &str, medium: Medium) -> io::Result<TapDesc> {
        let lower = unsafe {
            let lower = libc::open(
                c"/dev/net/tun".as_ptr() as *const libc::c_char,
                libc::O_RDWR | libc::O_NONBLOCK,
            );
            if lower == -1 {
                return Err(io::Error::last_os_error());
            }
            lower
        };

        let mut ifreq = ifreq_for(name);
        Self::attach_interface_ifreq(lower, &mut ifreq)?;
        log::trace!("Successfully attach interface: {}", name);
        let mtu = Self::mtu_ifreq(medium, &mut ifreq)?;
        log::trace!("Successfully get MTU: {}", mtu);
        let mac = Self::mac_ifreq(&mut ifreq)?;
        log::trace!("Successfully get MAC: {}", mac);

        Ok(TapDesc { lower, mtu, mac })
    }

    // pub fn from_fd(fd: RawFd, mtu: usize) -> io::Result<TapDesc> {
    //     Ok(TapDesc { lower: fd, mtu })
    // }

    fn attach_interface_ifreq(lower: libc::c_int, ifr: &mut ifreq) -> io::Result<()> {
        ifr.ifr_ifru.ifru_flags = (libc::IFF_TAP | libc::IFF_NO_PI) as libc::c_short;
        ifreq_ioctl(lower, libc::TUNSETIFF, ifr).map(|_| ())
    }

    fn mtu_ifreq(medium: Medium, ifr: &mut ifreq) -> io::Result<usize> {
        let lower = unsafe {
            let lower = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP);
            if lower == -1 {
                return Err(io::Error::last_os_error());
            }
            lower
        };
        defer!(unsafe {
            libc::close(lower);
        });

        let _ = ifreq_ioctl(lower, libc::SIOCGIFMTU, ifr)?;

        let ip_mtu = unsafe { ifr.ifr_ifru.ifru_mtu } as usize;

        // SIOCGIFMTU returns the IP MTU (typically 1500 bytes.)
        // smoltcp counts the entire Ethernet packet in the MTU, so add the Ethernet header size to it.
        let mtu = match medium {
            Medium::Ip => ip_mtu,
            Medium::Ethernet => ip_mtu + EthernetFrame::<&[u8]>::header_len(),
            // Medium::Ieee802154 => todo!(),
        };

        Ok(mtu)
    }

    fn mac_ifreq(ifr: &mut ifreq) -> io::Result<smoltcp::wire::EthernetAddress> {
        let lower = unsafe {
            let lower = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, libc::IPPROTO_IP);
            if lower == -1 {
                return Err(io::Error::last_os_error());
            }
            lower
        };
        defer!(unsafe {
            libc::close(lower);
        });

        let mac = smoltcp::wire::EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);

        unsafe {
            ifr.ifr_ifru.ifru_hwaddr.sa_family = libc::ARPHRD_ETHER;
            ifr.ifr_ifru.ifru_hwaddr.sa_data[..6]
                .copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x01]);
        }

        let _ = ifreq_ioctl(lower, libc::SIOCSIFHWADDR, ifr)?;

        Ok(mac)
    }

    pub fn interface_mtu(&self) -> io::Result<usize> {
        Ok(self.mtu)
    }

    pub fn recv(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let len = libc::read(
                self.lower,
                buffer.as_mut_ptr() as *mut libc::c_void,
                buffer.len(),
            );
            if len == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(len as usize)
        }
    }

    pub fn send(&mut self, buffer: &[u8]) -> io::Result<usize> {
        unsafe {
            let len = libc::write(
                self.lower,
                buffer.as_ptr() as *const libc::c_void,
                buffer.len(),
            );
            if len == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(len as usize)
        }
    }
}

impl Drop for TapDesc {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.lower);
        }
    }
}

use std::cell::RefCell;
use std::rc::Rc;
use std::vec::Vec;

use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;

/// A virtual TUN (IP) or TAP (Ethernet) interface.
#[derive(Debug)]
pub struct TapDevice {
    lower: Rc<RefCell<crate::driver::TapDesc>>,
    mtu: usize,
    medium: Medium,
}

impl AsRawFd for TapDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.lower.borrow().as_raw_fd()
    }
}

impl TapDevice {
    /// Attaches to a TUN/TAP interface called `name`, or creates it if it does not exist.
    ///
    /// If `name` is a persistent interface configured with UID of the current user,
    /// no special privileges are needed. Otherwise, this requires superuser privileges
    /// or a corresponding capability set on the executable.
    pub fn new(name: &str, medium: Medium) -> io::Result<TapDevice> {
        let lower = crate::driver::TapDesc::new(name, medium)?;
        let mtu = lower.interface_mtu()?;
        Ok(TapDevice {
            lower: Rc::new(RefCell::new(lower)),
            mtu,
            medium,
        })
    }

    // /// Attaches to a TUN/TAP interface specified by file descriptor `fd`.
    // ///
    // /// On platforms like Android, a file descriptor to a tun interface is exposed.
    // /// On these platforms, a TunTapInterface cannot be instantiated with a name.
    // pub fn from_fd(fd: RawFd, medium: Medium, mtu: usize) -> io::Result<TapDevice> {
    //     let lower = crate::driver::TapDesc::from_fd(fd, mtu)?;
    //     Ok(TapDevice {
    //         lower: Rc::new(RefCell::new(lower)),
    //         mtu,
    //         medium,
    //     })
    // }

    pub fn mac(&self) -> smoltcp::wire::EthernetAddress {
        self.lower.borrow().mac
    }
}

impl Device for TapDevice {
    type RxToken<'a> = RxToken;
    type TxToken<'a> = TxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = self.mtu;
        caps.medium = self.medium;
        caps
    }

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut lower = self.lower.borrow_mut();
        let mut buffer = vec![0; self.mtu];
        match lower.recv(&mut buffer[..]) {
            Ok(size) => {
                buffer.resize(size, 0);
                let rx = RxToken { buffer };
                let tx = TxToken {
                    lower: self.lower.clone(),
                };
                Some((rx, tx))
            }
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => None,
            Err(err) => panic!("{}", err),
        }
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        Some(TxToken {
            lower: self.lower.clone(),
        })
    }
}

#[doc(hidden)]
pub struct RxToken {
    buffer: Vec<u8>,
}

impl phy::RxToken for RxToken {
    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        f(&self.buffer[..])
    }
}

#[doc(hidden)]
pub struct TxToken {
    lower: Rc<RefCell<crate::driver::TapDesc>>,
}

impl phy::TxToken for TxToken {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut lower = self.lower.borrow_mut();
        let mut buffer = vec![0; len];
        let result = f(&mut buffer);
        match lower.send(&buffer[..]) {
            Ok(_) => {}
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
                log::debug!("phy: tx failed due to WouldBlock")
            }
            Err(err) => panic!("{}", err),
        }
        result
    }
}
