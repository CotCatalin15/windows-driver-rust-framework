use crate::kmalloc::{GlobalKernelAllocator, KernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type Vec<
    T: Sized + TaggedObject,
    A: KernelAllocator + alloc::alloc::Allocator = GlobalKernelAllocator<T>,
> = alloc::vec::Vec<T, A>;

pub trait VecExt<T>
where
    T: Sized + TaggedObject,
{
    fn try_push(&mut self, value: T) -> anyhow::Result<()>;

    fn create() -> Vec<T, GlobalKernelAllocator<T>> {
        Vec::new_in(GlobalKernelAllocator::default())
    }
}

impl<T> VecExt<T> for Vec<T, GlobalKernelAllocator<T>>
where
    T: Sized + TaggedObject,
{
    fn try_push(&mut self, value: T) -> anyhow::Result<()> {
        self.try_reserve(1)
            .map_err(|_| anyhow::Error::msg("Failed to reserve vec for try_push"))?;
        self.push(value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
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
        let mut v = Vec::create();

        v.try_push(10)?;

        GlobalKernelAllocator::<()>::fail_allocations(true);
        assert!(v.try_push(20).is_err());

        assert_eq!(v, [10]);

        Ok(())
    }
}
