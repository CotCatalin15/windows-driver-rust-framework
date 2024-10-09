use core::{cell::UnsafeCell, slice::SliceIndex};

use windows_sys::Wdk::System::SystemServices::{
    KeAcquireInStackQueuedSpinLock, KeInitializeSpinLock, KeReleaseInStackQueuedSpinLock,
};

use crate::traits::DispatchSafe;

pub struct StackSpinMutex<T: Send + DispatchSafe> {
    lock: UnsafeCell<usize>,
    inner: UnsafeCell<T>,
}

impl<T> StackSpinMutex<T>
where
    T: Send + DispatchSafe,
{
    pub fn new(data: T) -> Self {
        let handle = unsafe {
            let mut handle = 0;
            KeInitializeSpinLock(&mut handle);
            handle
        };

        Self {
            lock: UnsafeCell::new(handle),
            inner: UnsafeCell::new(data),
        }
    }

    pub fn acquire<R, F: FnOnce(&T) -> R>(&self, callback: F) -> R {
        unsafe {
            let mut handle = core::mem::zeroed();

            KeAcquireInStackQueuedSpinLock(self.lock.get(), &mut handle);
            let ret = callback(&*self.inner.get());
            KeReleaseInStackQueuedSpinLock(&handle);

            ret
        }
    }

    pub fn acquire_mut<R, F: FnOnce(&mut T) -> R>(&self, callback: F) -> R {
        unsafe {
            let mut handle = core::mem::zeroed();

            KeAcquireInStackQueuedSpinLock(self.lock.get(), &mut handle);
            let ret = callback(&mut *self.inner.get());
            KeReleaseInStackQueuedSpinLock(&handle);

            ret
        }
    }
}
