use core::cell::UnsafeCell;

use windows_sys::Wdk::System::SystemServices::{
    ExAcquireSpinLockExclusive, ExAcquireSpinLockShared, ExReleaseSpinLockExclusive,
    ExReleaseSpinLockShared,
};

use crate::traits::DispatchSafe;

use super::guard::{MutexGuard, ReadMutexGuard, Unlockable};

pub struct ExSpinMutex<T: DispatchSafe> {
    mutex: UnsafeCell<i32>,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send + DispatchSafe> Send for ExSpinMutex<T> {}
unsafe impl<T: Sync + DispatchSafe> Sync for ExSpinMutex<T> {}

impl<T> ExSpinMutex<T>
where
    T: DispatchSafe,
{
    pub fn new(data: T) -> Self {
        Self {
            mutex: UnsafeCell::new(0),
            inner: UnsafeCell::new(data),
        }
    }

    pub fn write(&self) -> MutexGuard<ExSpinWriteUnlockable<'_, T>> {
        let old_irql = unsafe { ExAcquireSpinLockExclusive(self.mutex.get()) };

        MutexGuard::new(ExSpinWriteUnlockable::new(self, old_irql), unsafe {
            &mut *self.inner.get()
        })
    }

    pub fn read(&self) -> ReadMutexGuard<ExSpinReadUnlockable<'_, T>> {
        let old_irql = unsafe { ExAcquireSpinLockShared(self.mutex.get()) };

        ReadMutexGuard::new(ExSpinReadUnlockable::new(self, old_irql), unsafe {
            &mut *self.inner.get()
        })
    }
}

pub struct ExSpinWriteUnlockable<'a, T: DispatchSafe> {
    guard: &'a ExSpinMutex<T>,
    old_irql: u8,
}

unsafe impl<'a, T> Send for ExSpinWriteUnlockable<'a, T> where T: Send + DispatchSafe {}

impl<'a, T: DispatchSafe> ExSpinWriteUnlockable<'a, T> {
    fn new(guard: &'a ExSpinMutex<T>, old_irql: u8) -> Self {
        Self { guard, old_irql }
    }
}

impl<'a, T: DispatchSafe> Unlockable for ExSpinWriteUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            ExReleaseSpinLockExclusive(self.guard.mutex.get(), self.old_irql);
        }
    }
}

pub struct ExSpinReadUnlockable<'a, T: DispatchSafe> {
    guard: &'a ExSpinMutex<T>,
    old_irql: u8,
}

unsafe impl<'a, T> Send for ExSpinReadUnlockable<'a, T> where T: Send + DispatchSafe {}

impl<'a, T: DispatchSafe> ExSpinReadUnlockable<'a, T> {
    fn new(guard: &'a ExSpinMutex<T>, old_irql: u8) -> Self {
        Self { guard, old_irql }
    }
}

impl<'a, T: DispatchSafe> Unlockable for ExSpinReadUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            ExReleaseSpinLockShared(self.guard.mutex.get(), self.old_irql);
        }
    }
}
