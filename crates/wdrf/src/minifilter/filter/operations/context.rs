use core::{
    any::{self, Any},
    ops::{Deref, DerefMut},
};

use wdrf_std::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag},
};

/*
This can be implemented by keeping track of the tag etc etc i dont have time etc etc
*/

#[repr(transparent)]
pub struct PostOpContext<T: ?Sized + 'static>(Box<T>);

impl<T: Sized> PostOpContext<T> {
    pub fn try_create(value: T) -> anyhow::Result<Self> {
        Box::try_create_in(value, Self::create_allocator()).map(|b| Self(b))
    }

    pub fn into_any(self) -> PostOpContext<dyn Any> {
        PostOpContext(self.0)
    }

    pub fn from_any(
        any: PostOpContext<dyn Any>,
    ) -> Result<PostOpContext<T>, PostOpContext<dyn Any>> {
        any.0
            .downcast()
            .map(|b| Self(b))
            .map_err(|e| PostOpContext(e))
    }
}

impl<T: ?Sized + 'static> PostOpContext<T> {
    pub(crate) fn leak(self) -> &'static mut T {
        Box::leak(self.0)
    }

    pub(crate) unsafe fn from_raw_ptr(raw: *mut T) -> Self {
        Self(Box::from_raw_in(raw, Self::create_allocator()))
    }

    fn get_generic_tag() -> MemoryTag {
        MemoryTag::new_from_bytes(b"fltc")
    }

    fn create_allocator() -> GlobalKernelAllocator {
        GlobalKernelAllocator::new(Self::get_generic_tag(), PoolFlags::POOL_FLAG_NON_PAGED)
    }
}

impl<T: ?Sized> Deref for PostOpContext<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: ?Sized> DerefMut for PostOpContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
