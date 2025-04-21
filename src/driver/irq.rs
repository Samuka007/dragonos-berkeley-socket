use std::os::unix::io::RawFd;
use std::time::Duration;
use std::{io, thread};

use hashbrown::HashMap;

use crate::socket::inet::common::NET_DEVICES;

const EPOLL_TIMEOUT_MS: i32 = 100;

/// Start a thread that polls network devices when their tap interfaces are readable
pub fn start_network_polling_thread() -> io::Result<thread::JoinHandle<()>> {
    // Create an epoll instance
    let epoll_fd = unsafe { libc::epoll_create1(0) };
    if epoll_fd < 0 {
        return Err(io::Error::last_os_error());
    }

    let handle = thread::spawn(move || {
        let mut events = Vec::with_capacity(32);
        events.resize(32, libc::epoll_event { events: 0, u64: 0 });
        let mut fd_to_device_id = HashMap::new();

        loop {
            // Update the list of devices to watch
            update_watched_devices(epoll_fd, &mut fd_to_device_id);

            // Wait for events
            let num_events = unsafe {
                libc::epoll_wait(
                    epoll_fd,
                    events.as_mut_ptr(),
                    events.len() as i32,
                    EPOLL_TIMEOUT_MS,
                )
            };

            if num_events < 0 {
                let err = io::Error::last_os_error();
                // Ignore EINTR, which happens when the thread receives a signal
                if err.kind() != io::ErrorKind::Interrupted {
                    log::error!("epoll_wait failed: {:?}", err);
                }
                continue;
            }

            // Process events
            for event in events.iter().take(num_events as usize) {
                let fd = event.u64 as RawFd;

                if let Some(&device_id) = fd_to_device_id.get(&fd) {
                    if let Some(device) = NET_DEVICES.read().get(&device_id) {
                        // Poll the device that has data available
                        device.poll();
                    }
                }
            }

            // Also poll all devices periodically regardless of events
            // This ensures timers and other internal state are updated
            for device in NET_DEVICES.read().values() {
                device.poll();
            }

            // Small sleep to prevent CPU hogging
            thread::sleep(Duration::from_millis(1));
        }
    });

    Ok(handle)
}

/// Update the list of devices being watched by epoll
fn update_watched_devices(epoll_fd: RawFd, fd_to_device_id: &mut HashMap<RawFd, usize>) {
    let devices = NET_DEVICES.read();

    // Find new devices to add
    for (&id, device) in devices.iter() {
        // Try to downcast to get TAP device
        if let Some(tap_fd) = device.raw_fd() {
            if !fd_to_device_id.contains_key(&tap_fd) {
                // Add new device to epoll
                let mut event = libc::epoll_event {
                    events: (libc::EPOLLIN | libc::EPOLLET) as u32,
                    u64: tap_fd as u64,
                };

                let result =
                    unsafe { libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, tap_fd, &mut event) };

                if result == 0 {
                    fd_to_device_id.insert(tap_fd, id);
                } else {
                    log::error!(
                        "Failed to add device fd {} to epoll: {:?}",
                        tap_fd,
                        io::Error::last_os_error()
                    );
                }
            }
        }
    }

    // Find devices to remove
    let mut to_remove = Vec::new();
    for &fd in fd_to_device_id.keys() {
        let id = fd_to_device_id[&fd];
        if !devices.contains_key(&id) {
            to_remove.push(fd);

            // Remove from epoll
            let result =
                unsafe { libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_DEL, fd, std::ptr::null_mut()) };

            if result != 0 {
                log::error!(
                    "Failed to remove device fd {} from epoll: {:?}",
                    fd,
                    io::Error::last_os_error()
                );
            }
        }
    }

    // Actually remove the entries
    for fd in to_remove {
        fd_to_device_id.remove(&fd);
    }
}
