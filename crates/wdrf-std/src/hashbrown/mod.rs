use allocator_api2::alloc::Allocator;

pub use hashbrown::hash_map::OccupiedError;

use crate::kmalloc::{GlobalKernelAllocator, TaggedObject};
pub type DefaultHashBuilder = core::hash::BuildHasherDefault<ahash::AHasher>;

#[allow(type_alias_bounds)]
pub type HashMap<K, V, S = DefaultHashBuilder, A: Allocator = GlobalKernelAllocator> =
    hashbrown::HashMap<K, V, S, A>;

pub trait HashMapExt<K, V, S = DefaultHashBuilder, A: Allocator = GlobalKernelAllocator> {
    fn create() -> HashMap<K, V, S, A>;
}

impl<K, V> HashMapExt<K, V, DefaultHashBuilder, GlobalKernelAllocator>
    for HashMap<K, V, DefaultHashBuilder, GlobalKernelAllocator>
where
    K: TaggedObject,
{
    fn create() -> HashMap<K, V, DefaultHashBuilder, GlobalKernelAllocator> {
        HashMap::with_hasher_in(
            DefaultHashBuilder::default(),
            GlobalKernelAllocator::new_for_tagged::<K>(),
        )
    }
}
