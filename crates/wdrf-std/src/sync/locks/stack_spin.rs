use core::cell::UnsafeCell;

use windows_sys::Wdk::System::SystemServices::{
    KeAcquireInStackQueuedSpinLock, KeInitializeSpinLock, KeReleaseInStackQueuedSpinLock,
    KLOCK_QUEUE_HANDLE,
};

#[cfg(feature = "irql-checks")]
use windows_sys::Wdk::System::SystemServices::DISPATCH_LEVEL;

use wdrf_macros::irql_check;

use crate::traits::DispatchSafe;

use super::guard::{MutexGuard, Unlockable};

pub struct InStackLockHandle {
    handle: UnsafeCell<KLOCK_QUEUE_HANDLE>,
}

pub struct StackSpinMutex<T: DispatchSafe> {
    lock: UnsafeCell<usize>,
    inner: UnsafeCell<T>,
}

impl<T> StackSpinMutex<T>
where
    T: DispatchSafe,
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

    #[cfg_attr(feature = "irql-checks", irql_check(irql = DISPATCH_LEVEL))]
    pub fn lock<'a>(
        &'a self,
        handle: &'a InStackLockHandle,
    ) -> MutexGuard<InStackSpinLockUnlocakble<'a, T>> {
        unsafe {
            self.lock_unchecked(handle);
        }

        MutexGuard::new(InStackSpinLockUnlocakble::new(self, handle), unsafe {
            &mut *self.inner.get()
        })
    }

    unsafe fn lock_unchecked<'a>(&'a self, handle: &'a InStackLockHandle) {
        unsafe {
            KeAcquireInStackQueuedSpinLock(self.lock.get(), &mut *handle.handle.get());
        }
    }

    unsafe fn unlock_unchecked(&self, handle: &InStackLockHandle) {
        unsafe {
            KeReleaseInStackQueuedSpinLock(&mut *handle.handle.get());
        }
    }
}

impl InStackLockHandle {
    pub fn new() -> Self {
        Self {
            handle: unsafe { core::mem::zeroed() },
        }
    }
}

pub struct InStackSpinLockUnlocakble<'a, T: DispatchSafe> {
    mutex: &'a StackSpinMutex<T>,
    handle: &'a InStackLockHandle,
}

unsafe impl<'a, T> Send for InStackSpinLockUnlocakble<'a, T> where T: Send + DispatchSafe {}

impl<'a, T> InStackSpinLockUnlocakble<'a, T>
where
    T: DispatchSafe,
{
    pub fn new(mutex: &'a StackSpinMutex<T>, handle: &'a InStackLockHandle) -> Self {
        Self { mutex, handle }
    }
}

impl<'a, T> Unlockable for InStackSpinLockUnlocakble<'a, T>
where
    T: DispatchSafe,
{
    type Item = T;

    fn unlock(&self) {
        unsafe {
            self.mutex.unlock_unchecked(self.handle);
        }
    }
}
