use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use crate::{
    sys,
    traits::{DispatchSafe, WriteLock},
};

pub type GuardedMutex<T> = Mutex<T, sys::mutex::GuardedMutex>;
pub type SpinMutex<T> = Mutex<T, sys::mutex::SpinLock>;

/// Its implemented using a guard mutex
/// It can be used at IRQL <= APC
///
pub struct Mutex<T: ?Sized, L: WriteLock> {
    inner: L,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, L: WriteLock> Send for Mutex<T, L> {}
unsafe impl<T: ?Sized + Send, L: WriteLock> Sync for Mutex<T, L> {}
unsafe impl<T: ?Sized + Send, L: WriteLock + DispatchSafe> DispatchSafe for Mutex<T, L> {}

pub struct MutexGuard<'a, T: ?Sized, L: WriteLock> {
    lock: &'a Mutex<T, L>,
}

impl<T, L> Mutex<T, L>
where
    L: WriteLock,
{
    pub fn new_in(data: T, lock: L) -> Self {
        Self {
            inner: lock,
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T, L> {
        self.inner.lock();
        MutexGuard::new(self)
    }
}

impl<T, L> Mutex<T, L>
where
    L: WriteLock + Default,
{
    pub fn new(data: T) -> Self {
        Self {
            inner: L::default(),
            data: UnsafeCell::new(data),
        }
    }
}

impl<'a, T, L> MutexGuard<'a, T, L>
where
    L: WriteLock,
{
    fn new(lock: &'a Mutex<T, L>) -> Self {
        Self { lock: lock }
    }
}

impl<T: ?Sized, L> Deref for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized, L> DerefMut for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized, L> Drop for MutexGuard<'_, T, L>
where
    L: WriteLock,
{
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}
