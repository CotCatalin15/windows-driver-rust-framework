use core::cell::UnsafeCell;

use wdk_sys::{
    ntddk::{KeInitializeSemaphore, KeReleaseSemaphore},
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

pub struct Semaphore(UnsafeCell<KSEMAPHORE>);

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}
unsafe impl DispatchSafe for Semaphore {}

impl TaggedObject for Semaphore {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"sema")
    }
}

pub struct SemaphoreGuard<'a> {
    owned: &'a Semaphore,
}

unsafe impl<'a> Send for SemaphoreGuard<'a> {}
unsafe impl<'a> Sync for SemaphoreGuard<'a> {}
unsafe impl<'a> DispatchSafe for SemaphoreGuard<'a> {}

impl<'a> SemaphoreGuard<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { owned: semaphore }
    }
}

impl<'a> Drop for SemaphoreGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            KeReleaseSemaphore(self.owned.0.get(), IO_NO_INCREMENT as _, 1 as _, false as _);
        }
    }
}

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
                Arc::try_create(Self(UnsafeCell::new(semaphore)))
            }
        }
    }

    //TODO: Add custome error, why failed
    pub fn acquire<'a>(&'a self) -> anyhow::Result<SemaphoreGuard<'a>> {
        if WaitResponse::Success != self.wait() {
            Err(anyhow::Error::msg("Failed to acquire semaphore"))
        } else {
            Ok(SemaphoreGuard::new(&self))
        }
    }
}

unsafe impl WaitableObject for Semaphore {
    unsafe fn get_object(&self) -> *const () {
        self.0.get().cast()
    }
}
