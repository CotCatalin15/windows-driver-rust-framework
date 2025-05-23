use wdrf_std::{
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag},
    sync::{InStackLockHandle, StackSpinMutex},
    vec::{Vec, VecCreate, VecExt},
};

pub struct LoggerAllocator {
    start_buffer_size: usize,
    free_buffers: StackSpinMutex<Vec<Vec<u8>>>,
}

impl LoggerAllocator {
    pub fn new(min_buffer_size: usize) -> Self {
        Self {
            start_buffer_size: min_buffer_size,
            free_buffers: StackSpinMutex::new(Vec::new_in(GlobalKernelAllocator::new(
                MemoryTag::new_from_bytes(b"fbfs"),
                PoolFlags::POOL_FLAG_NON_PAGED,
            ))),
        }
    }

    pub fn try_allocate(&self) -> anyhow::Result<Vec<u8>> {
        let handle = InStackLockHandle::new();
        let mut guard = self.free_buffers.lock(&handle);

        if !guard.is_empty() {
            Ok(guard.swap_remove(0))
        } else {
            core::mem::drop(guard);

            let mut buffer = Vec::create();
            buffer.try_resize(self.start_buffer_size, 0)?;

            Ok(buffer)
        }
    }

    pub fn free_allocation(&self, buf: Vec<u8>) {
        if buf.len() == self.start_buffer_size {
            let handle = InStackLockHandle::new();
            let mut guard = self.free_buffers.lock(&handle);
            let _ = guard.try_push(buf);
        }
    }
}
