use core::{cell::UnsafeCell, time::Duration};

use wdk_sys::{
    ntddk::{KeInitializeSemaphore, KeReadStateSemaphore, KeReleaseSemaphore},
    IO_NO_INCREMENT, KSEMAPHORE, PASSIVE_LEVEL, STATUS_SUCCESS,
};

use crate::{
    kmalloc::TaggedObject,
    sync::arc::{Arc, ArcExt},
    traits::DispatchSafe,
};

#[cfg(feature = "irql-checks")]
use wdrf_macros::irql_check;

use super::{WaitResponse, WaitableObject};

pub struct Semaphore {
    semaphore: UnsafeCell<KSEMAPHORE>,
    limit: u32,
}

unsafe impl DispatchSafe for Semaphore {}

impl TaggedObject for Semaphore {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"sema")
    }
}

pub struct SemaphoreGuard<'a> {
    owned: &'a Semaphore,
}

unsafe impl<'a> DispatchSafe for SemaphoreGuard<'a> {}

impl<'a> SemaphoreGuard<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { owned: semaphore }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            KeReleaseSemaphore(
                self.owned.semaphore.get(),
                IO_NO_INCREMENT as _,
                1 as _,
                false as _,
            );
        }
    }
}

#[derive(Clone)]
pub struct SemaphoreGuardOwned {
    owned: Arc<Semaphore>,
}

impl SemaphoreGuardOwned {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        Self { owned: semaphore }
    }
}

impl Drop for SemaphoreGuardOwned {
    fn drop(&mut self) {
        unsafe {
            KeReleaseSemaphore(
                self.owned.semaphore.get(),
                IO_NO_INCREMENT as _,
                1 as _,
                false as _,
            );
        }
    }
}

unsafe impl Send for SemaphoreGuardOwned {}
unsafe impl Sync for SemaphoreGuardOwned {}
unsafe impl DispatchSafe for SemaphoreGuardOwned {}

pub struct AcquireError;

impl Semaphore {
    #[cfg_attr(feature = "irql-check", irql_check(irql = PASSIVE_LEVEL))]
    pub fn new(limit: u32) -> anyhow::Result<Arc<Self>> {
        Self::new_in(0, limit)
    }

    #[cfg_attr(feature = "irql-check", irql_check(irql = PASSIVE_LEVEL))]
    pub fn new_in(count: u32, limit: u32) -> anyhow::Result<Arc<Self>> {
        if limit < count {
            Err(anyhow::Error::msg("Semaphore count > limit"))
        } else {
            unsafe {
                let mut semaphore = core::mem::zeroed();
                KeInitializeSemaphore(&mut semaphore, count as _, limit as _);
                Arc::try_create(Self {
                    semaphore: UnsafeCell::new(semaphore),
                    limit,
                })
            }
        }
    }

    pub fn acquire<'a>(&'a self) -> anyhow::Result<SemaphoreGuard<'a>, AcquireError> {
        if WaitResponse::Success != self.wait() {
            Err(AcquireError)
        } else {
            Ok(SemaphoreGuard::new(&self))
        }
    }

    pub fn try_acquire<'a>(&'a self) -> anyhow::Result<SemaphoreGuard<'a>, AcquireError> {
        if WaitResponse::Success != self.wait_for(Duration::from_nanos(0)) {
            Err(AcquireError)
        } else {
            Ok(SemaphoreGuard::new(&self))
        }
    }

    pub fn acquire_owned(self: Arc<Self>) -> anyhow::Result<SemaphoreGuardOwned, AcquireError> {
        if WaitResponse::Success != self.wait() {
            Err(AcquireError)
        } else {
            Ok(SemaphoreGuardOwned::new(self))
        }
    }

    pub fn try_acquire_owned(self: Arc<Self>) -> anyhow::Result<SemaphoreGuardOwned, AcquireError> {
        if WaitResponse::Success != self.wait_for(Duration::from_nanos(0)) {
            Err(AcquireError)
        } else {
            Ok(SemaphoreGuardOwned::new(self))
        }
    }

    pub fn read_state(&self) -> u32 {
        unsafe { KeReadStateSemaphore(self.semaphore.get()) as u32 }
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }
}

unsafe impl WaitableObject for Semaphore {
    unsafe fn get_object(&self) -> *const () {
        self.semaphore.get().cast()
    }
}
