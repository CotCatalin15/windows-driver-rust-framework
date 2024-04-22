use wdk_sys::{POOL_FLAG_NON_PAGED, UNICODE_STRING};

use crate::{
    kmalloc::{GlobalKernelAllocator, MemoryTag},
    vec::Vec,
};

pub struct UnicodeString {
    vec: Vec<u16>,
}

impl UnicodeString {
    pub fn create() -> Self {
        Self::create_in(GlobalKernelAllocator::new(
            MemoryTag::new_from_bytes(b"abcd"),
            POOL_FLAG_NON_PAGED,
        ))
    }

    pub fn create_in(alloc: GlobalKernelAllocator) -> Self {
        Self {
            vec: Vec::new_in(alloc),
        }
    }

    pub fn from_unicode(unicode: UNICODE_STRING) -> anyhow::Result<Self> {
        let str = Self::create();

        anyhow::bail!("Failed to create unicde");
    }
}
