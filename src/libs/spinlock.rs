pub use spin::{Mutex as SpinLock, MutexGuard};

// pub struct SpinLock<T: ?Sized> {
//     inner: Mutex<T>,
// }
// impl<T> SpinLock<T> {
//     pub fn new(data: T) -> Self {
//         Self {
//             inner: Mutex::new(data),
//         }
//     }

//     pub fn lock(&self) -> MutexGuard<'_, T> {
//         self.inner.lock()
//     }

//     pub fn lock_irq_disabled(&self) -> MutexGuard<'_, T> {
//         self.inner.lock()
//     }

//     pub fn lock_irqsave(&self) -> MutexGuard<'_, T> {
//         self.inner.lock()
//     }
// }
