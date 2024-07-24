use crate::traits::DispatchSafe;

use super::KernelObjectType;

use wdk_sys::{ntddk::ZwClose, HANDLE};

pub struct Handle {
    object_type: Option<KernelObjectType>,
    handle: HANDLE,
}

unsafe impl Send for Handle {}
unsafe impl Sync for Handle {}
unsafe impl DispatchSafe for Handle {}

impl Handle {
    pub fn new(obj_type: KernelObjectType, raw_handle: HANDLE) -> Self {
        Self {
            object_type: Some(obj_type),
            handle: raw_handle,
        }
    }

    pub unsafe fn new_unknown(raw_handle: HANDLE) -> Self {
        Self {
            object_type: None,
            handle: raw_handle,
        }
    }

    pub unsafe fn raw_handle(&self) -> HANDLE {
        self.handle
    }

    pub fn object_type(&self) -> Option<KernelObjectType> {
        self.object_type
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = ZwClose(self.handle);
        }
    }
}
