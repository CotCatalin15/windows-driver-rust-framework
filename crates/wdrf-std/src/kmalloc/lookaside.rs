use core::{alloc::Allocator, num::NonZeroU32, ptr::NonNull};

use wdk_sys::{
    ntddk::{
        self, ExAllocateFromLookasideListEx, ExFreeToLookasideListEx, ExInitializeLookasideListEx,
    },
    EX_LOOKASIDE_LIST_EX_FLAGS_FAIL_NO_RAISE, LOOKASIDE_LIST_EX, STATUS_NO_MEMORY,
    _POOL_TYPE::NonPagedPool,
};

use crate::{
    sync::arc::{Arc, ArcExt},
    NtResult, NtStatusError,
};

use super::{MemoryTag, TaggedObject};

#[derive(Clone)]
pub struct LookasideAllocator {
    list: Arc<LOOKASIDE_LIST_EX>,
    max_size: usize,
}

unsafe impl Send for LookasideAllocator {}
unsafe impl Sync for LookasideAllocator {}

impl TaggedObject for LOOKASIDE_LIST_EX {
    fn tag() -> super::MemoryTag {
        super::MemoryTag::new_from_bytes(b"lals")
    }
}

impl LookasideAllocator {
    pub fn new(size: NonZeroU32, tag: MemoryTag) -> NtResult<Self> {
        let mut list = Arc::try_create(unsafe { core::mem::zeroed::<LOOKASIDE_LIST_EX>() })
            .map_err(|e| NtStatusError::Status(STATUS_NO_MEMORY))?;

        let ptr: *const LOOKASIDE_LIST_EX = list.as_ref();
        let status = unsafe {
            ExInitializeLookasideListEx(
                ptr as _,
                core::mem::zeroed(),
                core::mem::zeroed(),
                NonPagedPool,
                EX_LOOKASIDE_LIST_EX_FLAGS_FAIL_NO_RAISE,
                size.get() as _,
                tag.tag() as _,
                0,
            )
        };

        if wdk::nt_success(status) {
            Ok(Self {
                list,
                max_size: size.get() as _,
            })
        } else {
            Err(NtStatusError::Status(status))
        }
    }
}

unsafe impl Allocator for LookasideAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        if layout.size() > self.max_size {
            return Err(core::alloc::AllocError);
        }

        unsafe {
            let ptr: *const LOOKASIDE_LIST_EX = self.list.as_ref();
            let alloc_ptr: *mut core::ffi::c_void = ExAllocateFromLookasideListEx(ptr as _);

            if alloc_ptr.is_null() {
                Err(core::alloc::AllocError)
            } else {
                let ptr = NonNull::new_unchecked(alloc_ptr as *mut u8);

                Ok(NonNull::slice_from_raw_parts(ptr, self.max_size))
            }
        }
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        if layout.size() > self.max_size {
            panic!("Cannot deallocate a pointer that was not from the lookaside list");
        }

        unsafe {
            let list: *const LOOKASIDE_LIST_EX = self.list.as_ref();
            ExFreeToLookasideListEx(list as _, ptr.as_ptr().cast());
        }
    }
}
