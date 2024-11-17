use core::cell::UnsafeCell;

use wdrf_std::{boxed::Box, NtResult, NtResultEx};
use windows_sys::Wdk::{
    Foundation::DRIVER_OBJECT,
    Storage::FileSystem::Minifilters::{
        FltRegisterFilter, FltStartFiltering, FltUnregisterFilter, FLT_REGISTRATION, PFLT_FILTER,
    },
};

use crate::context::Context;

use super::{FilterOperationVisitor, FltPostOpCallback, FltPreOpCallback};

pub struct MinifilterFramework {
    pub(crate) pre_operations: Box<dyn FltPreOpCallback>,
    pub(crate) post_operations: Box<dyn FltPostOpCallback>,
    pub(crate) filter_operations: Box<dyn FilterOperationVisitor>,
    pub(crate) filter: UnsafeCell<PFLT_FILTER>,
}

unsafe impl Send for MinifilterFramework {}
unsafe impl Sync for MinifilterFramework {}

pub(crate) static GLOBAL_MINIFILTER: Context<MinifilterFramework> = Context::uninit();

impl MinifilterFramework {
    pub(crate) fn new(
        pre_operations: Box<dyn FltPreOpCallback>,
        post_operations: Box<dyn FltPostOpCallback>,
        filter_operations: Box<dyn FilterOperationVisitor>,
    ) -> Self {
        Self {
            pre_operations,
            post_operations,
            filter_operations,
            filter: UnsafeCell::new(0),
        }
    }

    pub(crate) unsafe fn register_filter(
        &mut self,
        driver: *const DRIVER_OBJECT,
        registration: FLT_REGISTRATION,
    ) -> NtResult<()> {
        let status = FltRegisterFilter(driver, &registration, self.filter.get());

        NtResult::from_status(status, || ())
    }

    pub unsafe fn start_filtering() -> NtResult<()> {
        let status = FltStartFiltering(*GLOBAL_MINIFILTER.get().filter.get());

        NtResult::from_status(status, || ())
    }

    pub fn unregister() {
        unsafe {
            let framework = GLOBAL_MINIFILTER.get();
            if *framework.filter.get() != 0 {
                FltUnregisterFilter(*framework.filter.get());
                *framework.filter.get() = 0;
            }
        }
    }

    pub fn raw_filter(&self) -> PFLT_FILTER {
        unsafe { *self.filter.get() }
    }
}

impl Drop for MinifilterFramework {
    fn drop(&mut self) {
        Self::unregister();
    }
}
