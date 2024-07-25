use core::{alloc::Allocator, pin::Pin};

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};

#[allow(type_alias_bounds)]
pub type Box<T: ?Sized, A: Allocator = GlobalKernelAllocator> = alloc::boxed::Box<T, A>;

pub trait BoxExt<T> {
    fn try_create_in<A>(value: T, allocator: A) -> anyhow::Result<Box<T, A>>
    where
        A: Allocator;
    fn try_pin_in<A>(value: T, allocator: A) -> anyhow::Result<Pin<Box<T, A>>>
    where
        A: Allocator + 'static;

    fn try_create(value: T) -> anyhow::Result<Box<T, GlobalKernelAllocator>>
    where
        T: TaggedObject,
    {
        Self::try_create_in(value, GlobalKernelAllocator::new_for_tagged::<T>())
    }

    fn try_pin(value: T) -> anyhow::Result<Pin<Box<T, GlobalKernelAllocator>>>
    where
        T: TaggedObject,
    {
        Self::try_pin_in(value, GlobalKernelAllocator::new_for_tagged::<T>())
    }
}

impl<T> BoxExt<T> for Box<T> {
    fn try_create_in<A>(value: T, allocator: A) -> anyhow::Result<Box<T, A>>
    where
        A: Allocator,
    {
        Box::try_new_in(value, allocator).map_err(|_| anyhow::Error::msg("Failed to create box"))
    }

    fn try_pin_in<A>(value: T, allocator: A) -> anyhow::Result<Pin<Box<T, A>>>
    where
        A: Allocator + 'static,
    {
        let b = Box::try_create_in(value, allocator)?;

        Ok(b.into())
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
