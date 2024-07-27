use windows_sys::{Wdk::System::SystemServices::ZwClose, Win32::Foundation::HANDLE};

use crate::traits::DispatchSafe;

use super::KernelObjectType;

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

    ///
    /// # Safety
    ///
    /// Should not be used, it here just in case you do not know the type
    ///
    pub unsafe fn new_unknown(raw_handle: HANDLE) -> Self {
        Self {
            object_type: None,
            handle: raw_handle,
        }
    }

    ///
    /// # Safety
    ///
    /// Its safe to use as long as self lives longer than this
    /// If self drops the HANDLE becomes invalid which can cause problems
    ///
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
