use core::{cell::UnsafeCell, pin::Pin};

use windows_sys::Wdk::{
    Foundation::FAST_MUTEX,
    System::SystemServices::{
        ExAcquireSpinLockExclusive, ExAcquireSpinLockShared, ExReleaseSpinLockExclusive,
        ExReleaseSpinLockShared, KeAcquireGuardedMutex, KeAcquireInStackQueuedSpinLock,
        KeInitializeGuardedMutex, KeInitializeSpinLock, KeReleaseGuardedMutex,
        KeReleaseInStackQueuedSpinLock, KLOCK_QUEUE_HANDLE,
    },
};

use crate::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag},
    traits::{DispatchSafe, ReadLock, WriteLock},
};

pub struct GuardedMutex {
    inner: Pin<Box<UnsafeCell<FAST_MUTEX>>>,
}

unsafe impl Send for GuardedMutex {}
unsafe impl Sync for GuardedMutex {}

impl GuardedMutex {
    pub fn new() -> Self {
        unsafe {
            //TODO: use try_create instead of new for mutex creation
            let mut mutex = Box::try_pin_in(
                UnsafeCell::new(core::mem::zeroed()),
                GlobalKernelAllocator::new(
                    MemoryTag::new_from_bytes(b"gurd"),
                    PoolFlags::POOL_FLAG_NON_PAGED,
                ),
            )
            .unwrap();

            KeInitializeGuardedMutex(mutex.get_mut());

            Self { inner: mutex }
        }
    }
}

impl Default for GuardedMutex {
    fn default() -> Self {
        GuardedMutex::new()
    }
}

impl WriteLock for GuardedMutex {
    ///Irql < APC_LEVEL
    //#[cfg_attr(feature = "irql-checks", irql_check(irql = APC_LEVEL))]
    fn lock(&self) {
        unsafe {
            KeAcquireGuardedMutex(self.inner.get());
        }
    }

    ///Irql <= APC_LEVEL
    //#[cfg_attr(feature = "irql-checks", irql_check(irql = APC_LEVEL))]
    fn unlock(&self) {
        unsafe {
            KeReleaseGuardedMutex(self.inner.get());
        }
    }
}

pub struct SpinLock {
    inner: UnsafeCell<usize>,
    lock_handle: UnsafeCell<KLOCK_QUEUE_HANDLE>,
}

unsafe impl DispatchSafe for SpinLock {}

impl SpinLock {
    pub fn new() -> Self {
        unsafe {
            let mut mutex = 0;
            KeInitializeSpinLock(&mut mutex);
            Self {
                inner: UnsafeCell::new(mutex),
                lock_handle: UnsafeCell::new(core::mem::zeroed()),
            }
        }
    }
}

impl Default for SpinLock {
    fn default() -> Self {
        SpinLock::new()
    }
}

impl WriteLock for SpinLock {
    fn lock(&self) {
        unsafe {
            KeAcquireInStackQueuedSpinLock(self.inner.get(), self.lock_handle.get());
        }
    }

    fn unlock(&self) {
        unsafe {
            KeReleaseInStackQueuedSpinLock(self.lock_handle.get());
        }
    }
}

pub struct ExSpinLock {
    inner: UnsafeCell<i32>,
    old_irql: UnsafeCell<u8>,
}

unsafe impl DispatchSafe for ExSpinLock {}

impl ExSpinLock {
    pub const fn new() -> Self {
        unsafe {
            let mutex = core::mem::zeroed();
            Self {
                inner: UnsafeCell::new(mutex),
                old_irql: UnsafeCell::new(0),
            }
        }
    }
}

impl Default for ExSpinLock {
    fn default() -> Self {
        ExSpinLock::new()
    }
}

impl WriteLock for ExSpinLock {
    //#[cfg_attr(feature = "irql-check", irql_check(irql = DISPATCH_LEVEL))]
    fn lock(&self) {
        unsafe {
            let old_irql = ExAcquireSpinLockExclusive(self.inner.get());
            *self.old_irql.get() = old_irql;
        }
    }

    //#[cfg_attr(feature = "irql-check", irql_check(irql = DISPATCH_LEVEL, compare = IrqlCompare::Eq))]
    fn unlock(&self) {
        unsafe {
            ExReleaseSpinLockExclusive(self.inner.get(), *self.old_irql.get());
        }
    }
}

impl ReadLock for ExSpinLock {
    //#[cfg_attr(feature = "irql-check", irql_check(irql = DISPATCH_LEVEL))]
    fn lock_shared(&self) {
        unsafe {
            let old_irql = ExAcquireSpinLockShared(self.inner.get());
            *self.old_irql.get() = old_irql;
        }
    }

    //#[cfg_attr(feature = "irql-check", irql_check(irql = DISPATCH_LEVEL, compare = IrqlCompare::Eq))]
    fn unlock_shared(&self) {
        unsafe {
            ExReleaseSpinLockShared(self.inner.get(), *self.old_irql.get());
        }
    }
}
