use core::alloc::Allocator;

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type Vec<T: TaggedObject, A: Allocator = GlobalKernelAllocator> = alloc::vec::Vec<T, A>;

pub trait VecExt<T, A: Allocator>
where
    T: TaggedObject,
{
    fn try_push(&mut self, value: T) -> anyhow::Result<()>;

    fn try_insert(&mut self, idx: usize, value: T) -> anyhow::Result<()>;

    fn try_resize(&mut self, size: usize, value: T) -> anyhow::Result<()>
    where
        T: Clone;

    fn create() -> Vec<T, GlobalKernelAllocator> {
        Vec::new_in(GlobalKernelAllocator::new_for_tagged::<T>())
    }
}

impl<T, A: Allocator> VecExt<T, A> for Vec<T, A>
where
    T: TaggedObject,
{
    fn try_resize(&mut self, new_len: usize, value: T) -> anyhow::Result<()>
    where
        T: Clone,
    {
        if new_len > self.len() {
            self.try_reserve(new_len - self.len())
                .map_err(|_| anyhow::Error::msg("Vec::try_reserver failed"))?;
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
        //PANIC: insert panics if index > self.len
        if idx > self.len() {
            Err(anyhow::Error::msg("Index out of bounds"))
        } else {
            self.try_reserve(1)
                .map_err(|_| anyhow::Error::msg("Failed to reserve vec for try_push"))?;

            self.insert(idx, value);
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use crate::kmalloc::GlobalKernelAllocator;

    use super::{Vec, VecExt};

    #[test]
    fn testing_try_push() -> anyhow::Result<()> {
        let mut v = Vec::create();
        v.try_push(10)?;
        v.try_push(20)?;

        assert_eq!(v, [10, 20]);

        Ok(())
    }

    #[test]
    fn test_fail_push() -> anyhow::Result<()> {
        let mut alloc = GlobalKernelAllocator::new_for_tagged::<i32>();
        alloc.fail_allocations(true);

        let mut v = Vec::new_in(alloc);

        assert!(v.try_push(20).is_err());
        assert_eq!(v, []);

        Ok(())
    }
}
