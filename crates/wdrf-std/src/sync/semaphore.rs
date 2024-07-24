use core::{num::NonZeroU32, pin::Pin};

use crate::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    sys::{semaphore::KeSemaphore, WaitResponse, WaitableObject},
    traits::DispatchSafe,
};

use super::arc::{Arc, ArcExt};

pub struct Semaphore {
    inner: KeSemaphore,
    limit: u32,
}

unsafe impl Send for Semaphore {}
unsafe impl Sync for Semaphore {}
unsafe impl DispatchSafe for Semaphore {}

impl TaggedObject for Semaphore {
    fn tag() -> crate::kmalloc::MemoryTag {
        //Default kernel memory tag
        crate::kmalloc::MemoryTag::new_from_bytes(b"semp")
    }
}

pub struct SemaphorePermit<'a> {
    owned: &'a Semaphore,
}

unsafe impl Send for SemaphorePermit<'_> {}
unsafe impl Sync for SemaphorePermit<'_> {}
unsafe impl DispatchSafe for SemaphorePermit<'_> {}

impl<'a> SemaphorePermit<'a> {
    fn new(semaphore: &'a Semaphore) -> Self {
        Self { owned: semaphore }
    }
}

impl Drop for SemaphorePermit<'_> {
    fn drop(&mut self) {
        unsafe {
            self.owned.inner.release(NonZeroU32::new_unchecked(1));
        }
    }
}

pub struct SemaphorePermitOwned {
    owned: Arc<Semaphore>,
}

impl SemaphorePermitOwned {
    fn new(semaphore: Arc<Semaphore>) -> Self {
        Self { owned: semaphore }
    }
}

impl Drop for SemaphorePermitOwned {
    fn drop(&mut self) {
        unsafe {
            self.owned.inner.release(NonZeroU32::new_unchecked(1));
        }
    }
}

pub struct AcquireError;

impl Semaphore {
    pub fn try_create_arc(count: u32, limit: u32) -> anyhow::Result<Arc<Self>> {
        let arc_sem = Arc::try_create(Self {
            inner: unsafe { KeSemaphore::new() },
            limit,
        })?;

        arc_sem.inner.init(count, limit);

        Ok(arc_sem)
    }

    pub fn try_create_box(count: u32, limit: u32) -> anyhow::Result<Pin<Box<Self>>> {
        let box_sem = Box::try_pin(Self {
            inner: unsafe { KeSemaphore::new() },
            limit,
        })?;

        box_sem.inner.init(count, limit);

        Ok(box_sem)
    }

    pub fn acquire<'a>(&'a self) -> anyhow::Result<SemaphorePermit<'a>, AcquireError> {
        if self.wait() != WaitResponse::Success {
            Err(AcquireError)
        } else {
            Ok(SemaphorePermit::new(self))
        }
    }

    pub fn try_acquire<'a>(&'a self) -> anyhow::Result<SemaphorePermit<'a>, AcquireError> {
        if self.wait_status() != WaitResponse::Success {
            Err(AcquireError)
        } else {
            Ok(SemaphorePermit::new(self))
        }
    }

    pub fn acquire_owned(self: &Arc<Self>) -> anyhow::Result<SemaphorePermitOwned, AcquireError> {
        if self.wait() != WaitResponse::Success {
            Err(AcquireError)
        } else {
            Ok(SemaphorePermitOwned::new(self.clone()))
        }
    }

    pub fn try_acquire_owned(
        self: &Arc<Self>,
    ) -> anyhow::Result<SemaphorePermitOwned, AcquireError> {
        if self.wait_status() != WaitResponse::Success {
            Err(AcquireError)
        } else {
            Ok(SemaphorePermitOwned::new(self.clone()))
        }
    }

    pub fn release(&self, increment: NonZeroU32) {
        self.inner.release(increment);
    }

    pub fn limit(&self) -> u32 {
        self.limit
    }
}

unsafe impl WaitableObject for Semaphore {
    #[inline]
    fn kernel_object(&self) -> &crate::sys::WaitableKernelObject {
        self.inner.kernel_object()
    }

    #[inline]
    fn wait(&self) -> crate::sys::WaitResponse {
        self.inner.wait()
    }

    #[inline]
    fn wait_for(&self, duration: core::time::Duration) -> crate::sys::WaitResponse {
        self.inner.wait_for(duration)
    }

    #[inline]
    fn wait_status(&self) -> crate::sys::WaitResponse {
        self.inner.wait_status()
    }
}
