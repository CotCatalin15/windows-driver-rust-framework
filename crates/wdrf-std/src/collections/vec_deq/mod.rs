use core::alloc::Allocator;

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type VecDeque<T, A: Allocator = GlobalKernelAllocator> =
    alloc::collections::vec_deque::VecDeque<T, A>;

pub trait VecDequeExt<T> {
    fn try_push_back(&mut self, value: T) -> anyhow::Result<()>;
    fn try_push_front(&mut self, value: T) -> anyhow::Result<()>;
}

pub trait VecDequeCreate<T> {
    fn create() -> Self;
}

impl<T: TaggedObject> VecDequeCreate<T> for VecDeque<T, GlobalKernelAllocator> {
    fn create() -> Self {
        VecDeque::new_in(GlobalKernelAllocator::new_for_tagged::<T>())
    }
}

impl<T, A: Allocator> VecDequeExt<T> for VecDeque<T, A> {
    fn try_push_back(&mut self, value: T) -> anyhow::Result<()> {
        self.try_reserve(1)
            .map_err(|_| anyhow::Error::msg("Failed to reserve space for VecDequeExt"))?;

        self.push_back(value);

        Ok(())
    }

    fn try_push_front(&mut self, value: T) -> anyhow::Result<()> {
        self.try_reserve(1)
            .map_err(|_| anyhow::Error::msg("Failed to reserve space for VecDequeExt"))?;

        self.push_front(value);

        Ok(())
    }
}
