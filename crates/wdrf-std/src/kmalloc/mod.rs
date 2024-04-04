use core::{
    alloc::{Allocator, Layout},
    marker::PhantomData,
    ptr::NonNull,
};

use thiserror::Error;
use wdk_sys::{
    ntddk::{ExAllocatePool2, ExFreePoolWithTag},
    POOL_FLAG_NON_PAGED,
};

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
}

pub trait TaggedObject {
    fn tag() -> MemoryTag {
        //Default kernel memory tag
        MemoryTag::new_from_bytes(b"dkmt")
    }
    fn flags() -> u64 {
        POOL_FLAG_NON_PAGED
    }
}

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

pub struct TagLayout {
    pub tag: MemoryTag,
    pub flags: u64,
    pub layout: Layout,
}

impl TagLayout {
    pub fn new<T: TaggedObject>() -> Self {
        Self {
            layout: Layout::new::<T>(),
            flags: T::flags(),
            tag: T::tag(),
        }
    }

    pub fn from_layout<T: TaggedObject>(layout: Layout) -> Self {
        Self {
            tag: T::tag(),
            flags: T::flags(),
            layout,
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.layout.size()
    }

    #[inline]
    pub fn tag(&self) -> u32 {
        self.tag.tag
    }
}

/// Allocated memory the size of the layout
///
/// # Safety
///
/// * `layout` must be a valid
///
pub unsafe fn alloc(layout: TagLayout) -> *mut u8 {
    ExAllocatePool2(layout.flags, layout.layout.size() as u64, layout.tag()).cast()
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
pub unsafe fn dealloc(ptr: *mut u8, layout: TagLayout) {
    if ptr.is_null() {
        #[cfg(feature = "alloc-sanity")]
        {
            panic!("delloc provided with a null ptr");
        }
    } else {
        ExFreePoolWithTag(ptr.cast(), layout.tag());
    }
}

///A general trait for a kernel allocator that can allocate and deallocate memory
///
/// # Safety
///
/// Safety comment :)
pub unsafe trait KernelAllocator {
    ///Allocated a block of memory with size specified in the layout
    ///
    fn allocate(&self, layout: TagLayout) -> anyhow::Result<NonNull<[u8]>, AllocError>;

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// * `ptr` must denote a block of memory currently allocated via this allocator, and
    /// * `layout` must fit  that block of memory.
    ///
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: TagLayout);
}

pub struct GlobalKernelAllocator<T: TaggedObject> {
    _phantom: PhantomData<T>,
}

#[cfg(not(test))]
impl<T: TaggedObject> GlobalKernelAllocator<T> {
    #[inline]
    fn allocate_internal(&self, layout: TagLayout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let size = layout.size();
            let ptr = alloc(layout);
            if ptr.is_null() {
                Err(AllocError::OutOfMemory)
            } else {
                let ptr = core::slice::from_raw_parts_mut(ptr, size);

                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    unsafe fn deallocate_internal(&self, ptr: NonNull<u8>, _layout: TagLayout) {
        ExFreePoolWithTag(ptr.as_ptr().cast(), T::tag().tag);
    }
}

#[cfg(test)]
static mut TEST_ALLOCATOR_FAIL_ALLOC: bool = false;

#[cfg(test)]
impl<T: TaggedObject> GlobalKernelAllocator<T> {
    #[inline]
    fn allocate_internal(&self, layout: TagLayout) -> Result<NonNull<[u8]>, AllocError> {
        extern crate std;
        unsafe {
            let size = layout.size();
            let ptr = if TEST_ALLOCATOR_FAIL_ALLOC {
                core::ptr::null_mut()
            } else {
                std::alloc::alloc(layout.layout)
            };
            if ptr.is_null() {
                Err(AllocError::OutOfMemory)
            } else {
                let ptr = core::slice::from_raw_parts_mut(ptr, size);

                Ok(NonNull::new_unchecked(ptr))
            }
        }
    }

    unsafe fn deallocate_internal(&self, ptr: NonNull<u8>, layout: TagLayout) {
        extern crate std;
        std::alloc::dealloc(ptr.as_ptr(), layout.layout);
    }

    pub fn fail_allocations(fail: bool) {
        unsafe { TEST_ALLOCATOR_FAIL_ALLOC = fail };
    }
}

impl<T: TaggedObject> Default for GlobalKernelAllocator<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

unsafe impl<T: TaggedObject> KernelAllocator for GlobalKernelAllocator<T> {
    #[inline]
    fn allocate(&self, layout: TagLayout) -> Result<NonNull<[u8]>, AllocError> {
        self.allocate_internal(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: TagLayout) {
        self.deallocate_internal(ptr, layout);
    }
}

unsafe impl<T: TaggedObject> Allocator for GlobalKernelAllocator<T> {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, core::alloc::AllocError> {
        let layout = TagLayout::from_layout::<T>(layout);
        self.allocate_internal(layout)
            .map_err(|_| core::alloc::AllocError)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        let layout = TagLayout::from_layout::<T>(layout);
        self.deallocate_internal(ptr, layout)
    }
}
