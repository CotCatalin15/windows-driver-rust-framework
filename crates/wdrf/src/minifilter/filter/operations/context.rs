use core::ops::{Deref, DerefMut};

use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::{GlobalKernelAllocator, TaggedObject},
};

/*
This can be implemented by keeping track of the tag etc etc i dont have time etc etc
*/

#[repr(transparent)]
pub struct PostOpContext<T: 'static + Send + TaggedObject>(Box<T>);

impl<T: 'static + Send + TaggedObject> PostOpContext<T> {
    pub fn try_create(value: T) -> anyhow::Result<Self> {
        Box::try_create_in(value, GlobalKernelAllocator::new_for_tagged::<T>()).map(|b| Self(b))
    }

    pub(crate) fn leak(self) -> &'static mut T {
        Box::leak(self.0)
    }

    pub(crate) unsafe fn from_raw_ptr(raw: *mut T) -> Self {
        Self(Box::from_raw_in(
            raw,
            GlobalKernelAllocator::new_for_tagged::<T>(),
        ))
    }

    pub fn unwrap(self) -> T {
        Box::into_inner(self.0)
    }
}

impl<T: 'static + Send + TaggedObject> Deref for PostOpContext<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T: 'static + Send + Sync + TaggedObject> DerefMut for PostOpContext<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}
