pub mod communication;
pub mod fs;
pub mod io;
pub mod security_descriptor;
pub mod structs;

use core::mem::MaybeUninit;

use structs::IRP_MJ_OPERATION_END;
use wdrf_std::{kmalloc::TaggedObject, nt_success, NtResult, NtResultEx};
use windows_sys::Wdk::{
    Foundation::DRIVER_OBJECT,
    Storage::FileSystem::Minifilters::{
        FltRegisterFilter, FltStartFiltering, FltUnregisterFilter, FLT_CONTEXT_END,
        FLT_CONTEXT_REGISTRATION, FLT_OPERATION_REGISTRATION, FLT_REGISTRATION,
        FLT_REGISTRATION_VERSION, PFLT_FILTER, PFLT_FILTER_UNLOAD_CALLBACK,
    },
};

pub struct FltRegistration(FLT_REGISTRATION);

pub struct FltContextRegistrationSlice {
    inner: &'static [FLT_CONTEXT_REGISTRATION],
}

pub struct FltOperationRegistrationSlice {
    inner: &'static [FLT_OPERATION_REGISTRATION],
}

pub struct FltFilter(PFLT_FILTER);

unsafe impl Send for FltFilter {}
unsafe impl Sync for FltFilter {}

impl TaggedObject for FltFilter {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fltf")
    }
}

impl FltFilter {
    pub fn new(filter: isize) -> Self {
        Self(filter)
    }

    pub fn as_handle(&self) -> isize {
        self.0
    }

    ///
    /// # Safety
    ///
    /// Should only be called once
    ///
    pub unsafe fn start_filtering(&self) -> anyhow::Result<()> {
        let status = FltStartFiltering(self.0);
        if nt_success(status) {
            Ok(())
        } else {
            Err(anyhow::Error::msg("Failed to start filtering"))
        }
    }
}

impl Drop for FltFilter {
    fn drop(&mut self) {
        unsafe {
            FltUnregisterFilter(self.0);
        }
    }
}

impl FltContextRegistrationSlice {
    pub fn new(
        context: &'static [FLT_CONTEXT_REGISTRATION],
    ) -> Option<FltContextRegistrationSlice> {
        if !context.is_empty() && context.last().unwrap().ContextType == FLT_CONTEXT_END as _ {
            Some(FltContextRegistrationSlice { inner: context })
        } else {
            None
        }
    }

    pub fn get(&self) -> &'static [FLT_CONTEXT_REGISTRATION] {
        self.inner
    }
}

impl FltOperationRegistrationSlice {
    pub fn new(
        context: &'static [FLT_OPERATION_REGISTRATION],
    ) -> Option<FltOperationRegistrationSlice> {
        if !context.is_empty() && context.last().unwrap().MajorFunction != IRP_MJ_OPERATION_END as _
        {
            Some(FltOperationRegistrationSlice { inner: context })
        } else {
            None
        }
    }

    pub fn get(&self) -> &'static [FLT_OPERATION_REGISTRATION] {
        self.inner
    }
}

#[derive(Default)]
pub struct FltRegistrationBuilder {
    flags: u32,
    context: Option<FltContextRegistrationSlice>,
    operations: Option<FltOperationRegistrationSlice>,
    unload_cb: PFLT_FILTER_UNLOAD_CALLBACK,
}

impl FltRegistrationBuilder {
    pub fn new() -> Self {
        Self {
            flags: 0,
            context: None,
            operations: None,
            unload_cb: None,
        }
    }

    pub fn contexts(mut self, context: FltContextRegistrationSlice) -> Self {
        self.context = Some(context);
        self
    }

    pub fn operations(mut self, op: FltOperationRegistrationSlice) -> Self {
        self.operations = Some(op);
        self
    }

    pub fn unload(mut self, unload: PFLT_FILTER_UNLOAD_CALLBACK) -> Self {
        self.unload_cb = unload;
        self
    }

    pub fn build(self) -> anyhow::Result<FltRegistration> {
        let mut registration: FLT_REGISTRATION = unsafe { MaybeUninit::zeroed().assume_init() };

        registration.Size = core::mem::size_of::<FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration = self
            .operations
            .map_or_else(core::ptr::null, |c| c.get().as_ptr());

        registration.ContextRegistration = self
            .context
            .map_or_else(core::ptr::null, |c| c.get().as_ptr());

        registration.FilterUnloadCallback = self.unload_cb;

        Ok(FltRegistration::new(registration))
    }
}

impl FltRegistration {
    fn new(registration: FLT_REGISTRATION) -> Self {
        Self(registration)
    }

    pub fn register_filter(&self, driver: &mut DRIVER_OBJECT) -> NtResult<FltFilter> {
        let mut filter = 0;
        let status = unsafe { FltRegisterFilter(driver as _, &self.0, &mut filter) };

        NtResult::from_status(status, || FltFilter(filter))
    }
}
