use std::{sync::atomic::AtomicBool, thread::sleep, time::Duration};

use linux_errnos::Errno;

#[derive(Debug)]
pub struct WaitQueue {
    // events: AtomicUsize,
    is_scheduled: AtomicBool,
}

pub fn wq_wait_event_interruptible<T: Fn() -> bool>(
    wait_queue: &WaitQueue,
    should_wake: T,
    _: Option<usize>,
) -> Result<(), Errno> {
    // Simulate waiting for an event
    if should_wake() {
        wait_queue
            .is_scheduled
            .store(false, std::sync::atomic::Ordering::SeqCst);
        return Ok(());
    }
    wait_queue
        .is_scheduled
        .store(false, std::sync::atomic::Ordering::SeqCst);
    loop {
        if wait_queue
            .is_scheduled
            .load(std::sync::atomic::Ordering::SeqCst)
            && should_wake()
        {
            return Ok(());
        } else {
            // simulate never schedule to this process
            sleep(Duration::from_millis(10));
        }
    }
}

impl WaitQueue {
    pub fn wakeup(&self) {
        self.is_scheduled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Default for WaitQueue {
    fn default() -> Self {
        Self {
            // events: AtomicUsize::new(0),
            is_scheduled: AtomicBool::new(false),
        }
    }
}
