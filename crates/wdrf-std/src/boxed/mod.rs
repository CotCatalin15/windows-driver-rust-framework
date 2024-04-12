use core::alloc::Allocator;

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type Box<T: TaggedObject + ?Sized, A: Allocator = GlobalKernelAllocator> =
    alloc::boxed::Box<T, A>;

pub trait BoxExt<T: TaggedObject, A: Allocator> {
    fn try_create(value: T) -> anyhow::Result<Box<T, A>>;
}

impl<T: TaggedObject> BoxExt<T, GlobalKernelAllocator> for Box<T, GlobalKernelAllocator> {
    fn try_create(value: T) -> anyhow::Result<Box<T, GlobalKernelAllocator>> {
        Box::try_new_in(value, GlobalKernelAllocator::new_for_tagged::<T>())
            .map_err(|_| anyhow::Error::msg("Failed to create box"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Box, BoxExt};

    #[test]
    fn test() -> anyhow::Result<()> {
        let mut b = Box::try_create(10)?;
        assert_eq!(*b, 10);

        *b = 20;
        assert_eq!(*b, 20);

        Ok(())
    }
}
