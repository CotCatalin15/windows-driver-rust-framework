use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

use wdk_sys::{
    ntddk::{
        KeAcquireGuardedMutex, KeAcquireGuardedMutexUnsafe, KeGetCurrentIrql,
        KeInitializeGuardedMutex, KeReleaseGuardedMutex,
    },
    APC_LEVEL, KGUARDED_MUTEX,
};

/// Its implemented using a guard mutex
/// It can be used at IRQL <= APC
///
pub struct Mutex<T: ?Sized> {
    inner: UnsafeCell<KGUARDED_MUTEX>,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: ?Sized> {
    lock: &'a Mutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        let mutant = unsafe {
            let mut mutant = core::mem::zeroed::<KGUARDED_MUTEX>();
            KeInitializeGuardedMutex(&mut mutant);
            mutant
        };

        Self {
            inner: UnsafeCell::new(mutant),
            data: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        #[cfg(feature = "irql-sanity")]
        unsafe {
            let irql = KeGetCurrentIrql();
            if irql > APC_LEVEL as _ {
                panic!("Current irql {irql} <= DISPATCH");
            }
        }

        unsafe {
            KeAcquireGuardedMutex(self.inner.get());
        }
        MutexGuard::new(self)
    }

    pub fn lock_apc(&self) -> MutexGuard<'_, T> {
        #[cfg(feature = "irql-sanity")]
        unsafe {
            let irql = KeGetCurrentIrql();
            if irql != APC_LEVEL as _ {
                panic!("Current irql {irql} != APC");
            }
        }

        unsafe {
            KeAcquireGuardedMutexUnsafe(self.inner.get());
        }
        MutexGuard::new(self)
    }
}

impl<'a, T> MutexGuard<'a, T> {
    fn new(lock: &'a Mutex<T>) -> Self {
        Self { lock: lock }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            KeReleaseGuardedMutex(self.lock.inner.get());
        }
    }
}
