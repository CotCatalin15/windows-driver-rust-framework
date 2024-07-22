use core::alloc::Allocator;

use crate::{
    kmalloc::{GlobalKernelAllocator, TaggedObject},
    traits::DispatchSafe,
};

#[allow(type_alias_bounds)]
pub type Arc<T, A: Allocator = GlobalKernelAllocator> = alloc::sync::Arc<T, A>;

#[allow(type_alias_bounds)]
pub type Weak<T: TaggedObject, A: Allocator = GlobalKernelAllocator> = alloc::sync::Weak<T, A>;

unsafe impl<T: DispatchSafe, A: Allocator> DispatchSafe for Arc<T, A> {}
unsafe impl<T: DispatchSafe, A: Allocator> DispatchSafe for Weak<T, A> {}

pub trait ArcExt<T> {
    fn try_create(data: T) -> anyhow::Result<Arc<T, GlobalKernelAllocator>>
    where
        T: TaggedObject;

    fn try_create_in<A>(data: T, allocator: A) -> anyhow::Result<Arc<T, A>>
    where
        A: Allocator;
}

impl<T> ArcExt<T> for Arc<T>
where
    T: TaggedObject,
{
    fn try_create(data: T) -> anyhow::Result<Arc<T, GlobalKernelAllocator>> {
        Arc::try_new_in(data, GlobalKernelAllocator::new_for_tagged::<T>())
            .map_err(|_| anyhow::Error::msg("Failed to allocate ArcInner<T>"))
    }

    fn try_create_in<A>(data: T, allocator: A) -> anyhow::Result<Arc<T, A>>
    where
        A: Allocator,
    {
        Arc::try_new_in(data, allocator)
            .map_err(|_| anyhow::Error::msg("Failed to allocate ArcInner<T>"))
    }
}

#[cfg(test)]
mod tests {
    use super::{Arc, ArcExt};

    extern crate std;
    #[test]
    fn test() -> anyhow::Result<()> {
        let a = Arc::try_create(10)?;

        let wa = Arc::downgrade(&a);

        let ac = wa.upgrade().unwrap();

        assert_eq!(*ac, 10);
        assert_eq!(*a, 10);

        Ok(())
    }
}
