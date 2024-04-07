use core::cell::UnsafeCell;

use wdk_sys::{
    ntddk::{
        KeAcquireGuardedMutex, KeAcquireSpinLockRaiseToDpc, KeInitializeGuardedMutex,
        KeInitializeSpinLock, KeReleaseGuardedMutex, KeReleaseSpinLock,
    },
    APC_LEVEL, KGUARDED_MUTEX, KSPIN_LOCK,
};

#[cfg(feature = "irql-checks")]
use wdrf_macros::irql_check;

use crate::traits::{DispatchSafe, WriteLock};

pub struct GuardedMutex {
    inner: UnsafeCell<KGUARDED_MUTEX>,
    #[cfg(feature = "mutex-checks")]
    is_locked: UnsafeCell<bool>,
}

unsafe impl Send for GuardedMutex {}
unsafe impl Sync for GuardedMutex {}

impl GuardedMutex {
    pub fn new() -> Self {
        unsafe {
            let mut mutex = core::mem::zeroed();
            KeInitializeGuardedMutex(&mut mutex);
            Self {
                inner: UnsafeCell::new(mutex),
                #[cfg(feature = "mutex-checks")]
                is_locked: UnsafeCell::new(false),
            }
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
    #[cfg_attr(feature = "irql-checks", irql_check(irql = APC_LEVEL))]
    fn lock(&self) {
        unsafe {
            KeAcquireGuardedMutex(self.inner.get());

            #[cfg(feature = "mutex-checks")]
            {
                *self.is_locked.get() = true;
            }
        }
    }

    ///Irql <= APC_LEVEL
    #[cfg_attr(feature = "irql-checks", irql_check(irql = APC_LEVEL))]
    #[cfg_attr(feature = "mutex-checks", must_use)]
    fn unlock(&self) {
        #[cfg(feature = "mutex-checks")]
        unsafe {
            if (*self.is_locked.get()) == false {
                panic!("Unlock called without calling lock first");
            }
            *self.is_locked.get() = false;
        }

        unsafe {
            KeReleaseGuardedMutex(self.inner.get());
        }
    }
}

pub struct SpinLock {
    inner: UnsafeCell<KSPIN_LOCK>,
    old_irql: UnsafeCell<u8>,
    #[cfg(feature = "mutex-checks")]
    is_locked: UnsafeCell<bool>,
}

unsafe impl DispatchSafe for SpinLock {}

impl SpinLock {
    pub fn new() -> Self {
        unsafe {
            let mut mutex = core::mem::zeroed();
            KeInitializeSpinLock(&mut mutex);
            Self {
                inner: UnsafeCell::new(mutex),
                old_irql: UnsafeCell::new(0),
                #[cfg(feature = "mutex-checks")]
                is_locked: UnsafeCell::new(false),
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
    #[cfg_attr(featur = "irql-check", irql_check(irql = DISPATCH_LEVEL))]
    fn lock(&self) {
        unsafe {
            let old_irql = KeAcquireSpinLockRaiseToDpc(self.inner.get());
            *self.old_irql.get() = old_irql;

            #[cfg(feature = "mutex-checks")]
            {
                *self.is_locked.get() = true;
            }
        }
    }

    #[cfg_attr(featur = "irql-check", irql_check(irql = DISPATCH_LEVEL, compare = IrqlCompare::Eq))]
    fn unlock(&self) {
        unsafe {
            #[cfg(feature = "mutex-checks")]
            {
                if (*self.is_locked.get()) == false {
                    panic!("Unlock called without calling lock first");
                }
                *self.is_locked.get() = false;
            }

            KeReleaseSpinLock(self.inner.get(), *self.old_irql.get());
        }
    }
}
