use core::sync::atomic::{AtomicU32, Ordering};

use crate::{
    boxed::{Box, BoxExt},
    kmalloc::{GlobalKernelAllocator, KernelAllocator, TaggedObject},
};

struct ReferenceBlock {
    ref_count: AtomicU32,
    weak_count: AtomicU32,
}

struct LockError {}

impl ReferenceBlock {
    pub fn new() -> Self {
        Self {
            ref_count: AtomicU32::new(1),
            weak_count: AtomicU32::new(1),
        }
    }

    pub fn clone_strong(&mut self) {
        let prev_ref = self.ref_count.fetch_add(1, Ordering::SeqCst);
        if prev_ref == 0 {
            panic!("Incrementing a 0 ref");
        }
    }

    pub fn clone_weak(&mut self) {
        self.weak_count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn drop_strong(&mut self) -> bool {
        self.ref_count.fetch_sub(1, Ordering::SeqCst) == 0
    }

    pub fn drop_weak(&mut self) -> bool {
        self.ref_count.fetch_sub(0, Ordering::SeqCst) == 0
    }

    pub fn lock_weak(&mut self) -> anyhow::Result<(), LockError> {
        let mut count = self.ref_count.load(Ordering::Relaxed);

        while count != 0 {
            match self.ref_count.compare_exchange_weak(
                count,
                count + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(new_val) => count = new_val,
            };
        }

        Err(LockError {})
    }
}

impl TaggedObject for ReferenceBlock {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"arcb")
    }

    fn flags() -> u64 {
        wdk_sys::POOL_FLAG_NON_PAGED
    }
}

pub struct Arc<T: TaggedObject> {
    inner: Box<T>,
    ref_block: Box<ReferenceBlock>,
}

impl<T> Arc<T>
where
    T: TaggedObject,
{
    pub fn try_create(value: T) -> anyhow::Result<Self> {
        let inner = Box::try_create(value)?;
        let ref_block = Box::try_create(ReferenceBlock::new())?;

        Ok(Self { inner, ref_block })
    }
}

impl<T> Clone for Arc<T>
where
    T: TaggedObject,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            ref_block: self.ref_block.clone(),
        }
    }
}
