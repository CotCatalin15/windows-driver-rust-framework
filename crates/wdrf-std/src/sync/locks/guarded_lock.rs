use core::cell::UnsafeCell;

use anyhow::Ok;
use windows_sys::Wdk::{
    Foundation::FAST_MUTEX,
    System::SystemServices::{KeInitializeGuardedMutex, KeReleaseGuardedMutex},
};

use crate::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag},
};

use super::guard::{MutexGuard, Unlockable};

pub struct GuardedMutex<T> {
    mutex: Box<FAST_MUTEX>,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for GuardedMutex<T> {}
unsafe impl<T: Sync> Sync for GuardedMutex<T> {}

impl<T> GuardedMutex<T> {
    pub fn new(data: T) -> anyhow::Result<Self> {
        unsafe {
            let mut mutex = Box::try_create_in(
                core::mem::zeroed::<FAST_MUTEX>(),
                GlobalKernelAllocator::new(
                    MemoryTag::new_from_bytes(b"fast"),
                    PoolFlags::POOL_FLAG_PAGED,
                ),
            )?;

            KeInitializeGuardedMutex(mutex.as_mut());

            Ok(Self {
                mutex,
                inner: UnsafeCell::new(data),
            })
        }
    }

    pub fn lock(&self) -> MutexGuard<GuardedUnlockable<'_, T>> {
        MutexGuard::new(GuardedUnlockable { guard: self }, unsafe {
            &mut *self.inner.get()
        })
    }
}

pub struct GuardedUnlockable<'a, T> {
    guard: &'a GuardedMutex<T>,
}

unsafe impl<'a, T> Send for GuardedUnlockable<'a, T> where T: Send {}

impl<'a, T> Unlockable for GuardedUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            let ptr: *const FAST_MUTEX = self.guard.mutex.as_ref();
            KeReleaseGuardedMutex(ptr as _);
        }
    }
}
