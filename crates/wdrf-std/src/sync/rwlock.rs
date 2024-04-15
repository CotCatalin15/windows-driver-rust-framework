use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use crate::{
    sys::mutex::ExSpinLock,
    traits::{DispatchSafe, ReadLock, WriteLock},
};

pub type ExRwLock<T> = RwLock<T, ExSpinLock>;

pub struct RwLock<T: ?Sized, L: WriteLock + ReadLock> {
    inner: L,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send, L: WriteLock + ReadLock> Send for RwLock<T, L> {}
unsafe impl<T: ?Sized + Send + Sync, L: WriteLock + ReadLock> Sync for RwLock<T, L> {}
unsafe impl<T: ?Sized + Send + Sync, L: WriteLock + ReadLock + DispatchSafe> DispatchSafe
    for RwLock<T, L>
{
}

#[must_use]
pub struct RwLockReadGuard<'a, T: ?Sized + 'a, L: WriteLock + ReadLock + 'a> {
    data: NonNull<T>,
    inner_lock: &'a L,
}

impl<T: ?Sized, L: ReadLock> !Send for RwLockReadGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: WriteLock + ReadLock> Sync for RwLockReadGuard<'_, T, L> {}

#[must_use]
pub struct RwLockWriteGuard<'a, T: ?Sized + 'a, L: ReadLock + WriteLock + 'a> {
    inner_lock: &'a RwLock<T, L>,
}

impl<T: ?Sized, L: WriteLock> !Send for RwLockWriteGuard<'_, T, L> {}
unsafe impl<T: ?Sized + Sync, L: ReadLock + WriteLock> Sync for RwLockWriteGuard<'_, T, L> {}

impl<T, L> RwLock<T, L>
where
    L: ReadLock + WriteLock + Default,
{
    pub fn new(data: T) -> Self {
        Self {
            inner: L::default(),
            data: UnsafeCell::new(data),
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, T, L> {
        self.inner.lock();
        RwLockWriteGuard::new(self)
    }

    pub fn read(&self) -> RwLockReadGuard<'_, T, L> {
        self.inner.lock_shared();
        RwLockReadGuard::new(self)
    }
}

impl<T, L> RwLock<T, L>
where
    L: ReadLock + WriteLock,
{
    pub fn new_in(data: T, lock: L) -> Self {
        Self {
            inner: lock,
            data: UnsafeCell::new(data),
        }
    }
}

impl<'a, T, L> RwLockReadGuard<'a, T, L>
where
    L: ReadLock + WriteLock,
{
    fn new(lock: &'a RwLock<T, L>) -> Self {
        unsafe {
            Self {
                data: NonNull::new_unchecked(lock.data.get()),
                inner_lock: &lock.inner,
            }
        }
    }
}

impl<'a, T, L> RwLockWriteGuard<'a, T, L>
where
    L: ReadLock + WriteLock,
{
    fn new(lock: &'a RwLock<T, L>) -> Self {
        Self { inner_lock: lock }
    }
}

impl<'a, T, L> Deref for RwLockWriteGuard<'a, T, L>
where
    T: ?Sized,
    L: ReadLock + WriteLock,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner_lock.data.get() }
    }
}

impl<'a, T, L> DerefMut for RwLockWriteGuard<'a, T, L>
where
    T: ?Sized,
    L: ReadLock + WriteLock,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner_lock.data.get() }
    }
}

impl<'a, T, L> Deref for RwLockReadGuard<'a, T, L>
where
    T: ?Sized,
    L: ReadLock + WriteLock,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.data.as_ref() }
    }
}

impl<'a, T, L> Drop for RwLockWriteGuard<'a, T, L>
where
    T: ?Sized,
    L: ReadLock + WriteLock,
{
    fn drop(&mut self) {
        self.inner_lock.inner.unlock();
    }
}

impl<'a, T, L> Drop for RwLockReadGuard<'a, T, L>
where
    T: ?Sized,
    L: ReadLock + WriteLock,
{
    fn drop(&mut self) {
        self.inner_lock.unlock_shared();
    }
}
