use core::num::NonZeroU32;

use wdk_sys::{
    ntddk::{KeInitializeSemaphore, KeReadStateSemaphore, KeReleaseSemaphore},
    IO_NO_INCREMENT, KSEMAPHORE,
};

use crate::kmalloc::TaggedObject;

use super::{WaitableKernelObject, WaitableObject};

#[repr(C)]
pub struct KeSemaphore(KSEMAPHORE);

impl TaggedObject for KeSemaphore {
    fn tag() -> crate::kmalloc::MemoryTag {
        //Default kernel memory tag
        crate::kmalloc::MemoryTag::new_from_bytes(b"kesm")
    }
}

impl KeSemaphore {
    ///
    ///# Safety
    ///
    /// Moving this object will invalidate internal pointers
    /// resulting in  a BugCheck
    ///
    pub unsafe fn new() -> Self {
        Self(unsafe { core::mem::zeroed() })
    }

    pub fn init(&self, count: u32, limit: u32) {
        unsafe {
            let ptr: *const KSEMAPHORE = &self.0;
            KeInitializeSemaphore(ptr as _, count as _, limit as _);
        }
    }

    pub fn release(&self, increment: NonZeroU32) {
        unsafe {
            let ptr: *const KSEMAPHORE = &self.0;
            KeReleaseSemaphore(
                ptr as _,
                IO_NO_INCREMENT as _,
                increment.get() as _,
                false as _,
            );
        }
    }

    pub fn read_state(&self) -> u32 {
        unsafe {
            let ptr: *const KSEMAPHORE = &self.0;
            KeReadStateSemaphore(ptr as _) as _
        }
    }
}

unsafe impl WaitableObject for KeSemaphore {
    fn kernel_object(&self) -> &WaitableKernelObject {
        unsafe {
            let ptr: *const KSEMAPHORE = &self.0;
            &*ptr.cast()
        }
    }
}
