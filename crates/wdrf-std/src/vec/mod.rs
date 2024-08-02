use core::alloc::Allocator;

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type Vec<T, A: Allocator = GlobalKernelAllocator> = alloc::vec::Vec<T, A>;

pub trait VecCreate<T> {
    fn create() -> Vec<T, GlobalKernelAllocator>
    where
        T: TaggedObject,
    {
        Vec::new_in(GlobalKernelAllocator::new_for_tagged::<T>())
    }
}

pub trait VecExt<T, A: Allocator> {
    fn try_push(&mut self, value: T) -> anyhow::Result<()>;

    fn try_insert(&mut self, idx: usize, value: T) -> anyhow::Result<()>;

    fn try_resize(&mut self, size: usize, value: T) -> anyhow::Result<()>
    where
        T: Clone;
}

impl<T> VecCreate<T> for Vec<T, GlobalKernelAllocator> {}

impl<T, A: Allocator> VecExt<T, A> for Vec<T, A> {
    fn try_resize(&mut self, new_len: usize, value: T) -> anyhow::Result<()>
    where
        T: Clone,
    {
        if new_len > self.len() {
            self.try_reserve(new_len - self.len())
                .map_err(|_| anyhow::Error::msg("Vec::try_reserve failed"))?;
        }

        self.resize(new_len, value);
        Ok(())
    }

    fn try_push(&mut self, value: T) -> anyhow::Result<()> {
        self.try_reserve(1)
            .map_err(|_| anyhow::Error::msg("Failed to reserve vec for try_push"))?;

        self.push(value);
        Ok(())
    }

    fn try_insert(&mut self, idx: usize, value: T) -> anyhow::Result<()> {
        // PANIC: insert panics if index > self.len
        if idx > self.len() {
            Err(anyhow::Error::msg("Index out of bounds"))
        } else {
            self.try_reserve(1)
                .map_err(|_| anyhow::Error::msg("Failed to reserve vec for try_insert"))?;

            self.insert(idx, value);
            Ok(())
        }
    }
}
