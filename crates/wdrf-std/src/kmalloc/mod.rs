use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    ptr::NonNull,
};

use thiserror::Error;
use windows_sys::Wdk::System::SystemServices::{ExAllocatePool2, ExFreePoolWithTag};

use crate::constants::PoolFlags;

#[derive(Clone, Copy)]
pub struct MemoryTag {
    tag: u32,
}

impl MemoryTag {
    pub const fn new(tag: u32) -> Self {
        Self { tag }
    }

    pub const fn new_from_bytes(b: &[u8; 4]) -> Self {
        Self {
            tag: u32::from_ne_bytes(*b),
        }
    }

    pub fn tag(&self) -> u32 {
        self.tag
    }
}

pub trait TaggedObject {
    fn tag() -> MemoryTag {
        //Default kernel memory tag
        MemoryTag::new_from_bytes(b"dkmt")
    }
    fn flags() -> PoolFlags {
        PoolFlags::POOL_FLAG_NON_PAGED
    }
}

impl TaggedObject for i8 {}
impl TaggedObject for u8 {}

impl TaggedObject for i16 {}
impl TaggedObject for u16 {}

impl TaggedObject for i32 {}
impl TaggedObject for u32 {}

impl TaggedObject for i64 {}
impl TaggedObject for u64 {}

impl TaggedObject for () {}

#[derive(Error, Debug)]
pub enum AllocError {
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Invalid flag {0}")]
    InvalidFlags(u32),
    #[error("Unknown error: {0}")]
    Unknown(&'static str),
}

/// Allocated memory the size of the layout
///
/// # Safety
///
/// * `layout` must be valid
///
pub unsafe fn alloc(tag: MemoryTag, flags: PoolFlags, layout: Layout) -> *mut u8 {
    ExAllocatePool2(flags.bits(), layout.size(), tag.tag).cast()
}

/// Deallocates the memory referenced by `ptr`.
///
/// # Safety
///
/// * `ptr` must denote a block of memory [*currently allocated*] via this allocator, and
/// * `layout` must [*fit*] that block of memory.
///
/// [*currently allocated*]: #currently-allocated-memory
/// [*fit*]: #memory-fitting
pub unsafe fn dealloc(ptr: *mut u8, tag: MemoryTag, _layout: Layout) {
    if ptr.is_null() {
        #[cfg(feature = "alloc-sanity")]
        {
            panic!("delloc provided with a null ptr");
        }
    } else {
        ExFreePoolWithTag(ptr.cast(), tag.tag);
    }
}

#[derive(Clone, Copy)]
pub struct GlobalKernelAllocator {
    tag: MemoryTag,
    flags: PoolFlags,

    #[cfg(test)]
    fail_alloc: bool,
}

unsafe impl Send for GlobalKernelAllocator {}
unsafe impl Sync for GlobalKernelAllocator {}

impl GlobalKernelAllocator {
    pub const fn new(tag: MemoryTag, flags: PoolFlags) -> Self {
        Self {
            tag,
            flags,
            #[cfg(test)]
            fail_alloc: false,
        }
    }

    pub fn new_for_tagged<T: TaggedObject>() -> Self {
        Self {
            tag: T::tag(),
            flags: T::flags(),
            #[cfg(test)]
            fail_alloc: false,
        }
    }
}

#[cfg(not(test))]
impl GlobalKernelAllocator {
    #[inline]
    fn allocate_internal(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let size = layout.size();
            let ptr = alloc(self.tag, self.flags, layout);
            if ptr.is_null() {
                Err(AllocError::OutOfMemory)
            } else {
                let ptr = core::slice::from_raw_parts_mut(ptr, size);

                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    unsafe fn deallocate_internal(&self, ptr: NonNull<u8>, layout: Layout) {
        dealloc(ptr.as_ptr(), self.tag, layout);
    }
}

#[cfg(test)]
impl GlobalKernelAllocator {
    #[inline]
    fn allocate_internal(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        extern crate std;
        unsafe {
            let size = layout.size();
            let ptr = if self.fail_alloc {
                std::println!("[Alloc] Failing to alloc size: {size}");
                core::ptr::null_mut()
            } else {
                std::println!("[Alloc] Allocating size: {size}");
                std::alloc::alloc(layout)
            };
            if ptr.is_null() {
                Err(AllocError::OutOfMemory)
            } else {
                let ptr = core::slice::from_raw_parts_mut(ptr, size);

                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    unsafe fn deallocate_internal(&self, ptr: NonNull<u8>, layout: Layout) {
        extern crate std;
        std::println!("Deallocating {}", layout.size());
        std::alloc::dealloc(ptr.as_ptr(), layout);
    }

    pub fn fail_allocations(&mut self, fail: bool) {
        self.fail_alloc = fail;
    }
}

unsafe impl Allocator for GlobalKernelAllocator {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        self.allocate_internal(layout)
            .map_err(|_| core::alloc::AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.deallocate_internal(ptr, layout);
    }
}

unsafe impl allocator_api2::alloc::Allocator for GlobalKernelAllocator {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, allocator_api2::alloc::AllocError> {
        self.allocate_internal(layout)
            .map_err(|_| allocator_api2::alloc::AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.deallocate_internal(ptr, layout);
    }
}

unsafe impl GlobalAlloc for GlobalKernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocate_internal(layout);

        match ptr {
            Ok(p) => p.as_ptr().as_mut_ptr(),
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate_internal(NonNull::new(ptr).unwrap(), layout);
    }
}
