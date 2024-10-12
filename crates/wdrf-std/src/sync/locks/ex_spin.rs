use core::cell::UnsafeCell;

use windows_sys::Wdk::System::SystemServices::{
    ExAcquireSpinLockExclusive, ExAcquireSpinLockShared, ExReleaseSpinLockExclusive,
    ExReleaseSpinLockShared,
};

use crate::traits::DispatchSafe;

use super::guard::{MutexGuard, ReadMutexGuard, Unlockable};

pub struct ExSpinMutex<T: Send + DispatchSafe> {
    mutex: UnsafeCell<i32>,
    inner: UnsafeCell<T>,
}

impl<T> ExSpinMutex<T>
where
    T: Send + DispatchSafe,
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

pub struct ExSpinWriteUnlockable<'a, T: Send + DispatchSafe> {
    guard: &'a ExSpinMutex<T>,
    old_irql: u8,
}

impl<'a, T: Send + DispatchSafe> ExSpinWriteUnlockable<'a, T> {
    fn new(guard: &'a ExSpinMutex<T>, old_irql: u8) -> Self {
        Self { guard, old_irql }
    }
}

impl<'a, T: Send + DispatchSafe> Unlockable for ExSpinWriteUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            ExReleaseSpinLockExclusive(self.guard.mutex.get(), self.old_irql);
        }
    }
}

pub struct ExSpinReadUnlockable<'a, T: Send + DispatchSafe> {
    guard: &'a ExSpinMutex<T>,
    old_irql: u8,
}

impl<'a, T: Send + DispatchSafe> ExSpinReadUnlockable<'a, T> {
    fn new(guard: &'a ExSpinMutex<T>, old_irql: u8) -> Self {
        Self { guard, old_irql }
    }
}

impl<'a, T: Send + DispatchSafe> Unlockable for ExSpinReadUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            ExReleaseSpinLockShared(self.guard.mutex.get(), self.old_irql);
        }
    }
}
