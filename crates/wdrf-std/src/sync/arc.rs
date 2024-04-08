use crate::{
    kmalloc::{GlobalKernelAllocator, KernelAllocator, TaggedObject},
    traits::DispatchSafe,
};

#[allow(type_alias_bounds)]
pub type Arc<
    T: TaggedObject,
    A: KernelAllocator + alloc::alloc::Allocator = GlobalKernelAllocator<T>,
> = alloc::sync::Arc<T, A>;

#[allow(type_alias_bounds)]
pub type Weak<
    T: TaggedObject,
    A: KernelAllocator + alloc::alloc::Allocator = GlobalKernelAllocator<T>,
> = alloc::sync::Weak<T, A>;

unsafe impl<T: TaggedObject + DispatchSafe, A: KernelAllocator + alloc::alloc::Allocator>
    DispatchSafe for Arc<T, A>
{
}

unsafe impl<T: TaggedObject + DispatchSafe, A: KernelAllocator + alloc::alloc::Allocator>
    DispatchSafe for Weak<T, A>
{
}

pub trait ArcExt<T, A>
where
    T: TaggedObject,
    A: KernelAllocator + alloc::alloc::Allocator,
{
    fn try_create(data: T) -> anyhow::Result<Arc<T, A>>;
}

impl<T> ArcExt<T, GlobalKernelAllocator<T>> for Arc<T, GlobalKernelAllocator<T>>
where
    T: TaggedObject,
{
    fn try_create(data: T) -> anyhow::Result<Arc<T, GlobalKernelAllocator<T>>> {
        Arc::try_new_in(data, GlobalKernelAllocator::<T>::default())
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

        std::println!("Arc value: {}", *ac);

        Ok(())
    }
}
