use std::{io, mem};

pub mod tap;
pub use tap::TapDesc;
pub mod irq;

use libc::ifreq;

fn ifreq_for(name: &str) -> ifreq {
    let mut ifreq = ifreq {
        ifr_name: [0; libc::IFNAMSIZ],
        ifr_ifru: unsafe { mem::zeroed() },
    };
    for (i, byte) in name.as_bytes().iter().enumerate() {
        ifreq.ifr_name[i] = *byte as libc::c_char
    }
    ifreq
}

fn ifreq_ioctl(lower: libc::c_int, cmd: libc::c_ulong, ifreq: &mut ifreq) -> io::Result<i32> {
    unsafe {
        let res = libc::ioctl(lower, cmd as _, ifreq as *mut ifreq);
        if res == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(res)
    }
}
