use core::{mem::MaybeUninit, ptr::NonNull};

use wdk_sys::{
    fltmgr::{
        FltRegisterFilter, FltUnloadFilter, FltUnregisterFilter, FLT_CONTEXT_END,
        FLT_REGISTRATION_VERSION, IRP_MJ_OPERATION_END, PFLT_FILTER_UNLOAD_CALLBACK,
        _FLT_CONTEXT_REGISTRATION, _FLT_FILTER, _FLT_OPERATION_REGISTRATION, _FLT_REGISTRATION,
    },
    DRIVER_OBJECT,
};
use wdrf_std::kmalloc::TaggedObject;

pub mod communication;

pub struct FltRegistration(_FLT_REGISTRATION);

pub struct FltContextRegistrationSlice {
    inner: &'static [_FLT_CONTEXT_REGISTRATION],
}

pub struct FltOperationRegistrationSlice {
    inner: &'static [_FLT_OPERATION_REGISTRATION],
}

pub struct FltFilter(NonNull<_FLT_FILTER>);

unsafe impl Send for FltFilter {}
unsafe impl Sync for FltFilter {}

impl TaggedObject for FltFilter {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fltf")
    }
}

impl FltFilter {
    pub fn new(filter: NonNull<_FLT_FILTER>) -> Self {
        Self(filter)
    }

    pub fn as_ptr(&self) -> NonNull<_FLT_FILTER> {
        self.0.clone()
    }
}

impl Drop for FltFilter {
    fn drop(&mut self) {
        unsafe {
            FltUnregisterFilter(self.0.as_ptr());
        }
    }
}

impl FltContextRegistrationSlice {
    pub fn new(
        context: &'static [_FLT_CONTEXT_REGISTRATION],
    ) -> Option<FltContextRegistrationSlice> {
        if context.is_empty() {
            None
        } else {
            if context.last().unwrap().ContextType != FLT_CONTEXT_END as _ {
                None
            } else {
                Some(FltContextRegistrationSlice { inner: context })
            }
        }
    }

    pub fn get(&self) -> &'static [_FLT_CONTEXT_REGISTRATION] {
        self.inner
    }
}

impl FltOperationRegistrationSlice {
    pub fn new(
        context: &'static [_FLT_OPERATION_REGISTRATION],
    ) -> Option<FltOperationRegistrationSlice> {
        if context.is_empty() {
            None
        } else {
            if context.last().unwrap().MajorFunction != IRP_MJ_OPERATION_END as _ {
                None
            } else {
                Some(FltOperationRegistrationSlice { inner: context })
            }
        }
    }

    pub fn get(&self) -> &'static [_FLT_OPERATION_REGISTRATION] {
        self.inner
    }
}

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
        let mut registration: _FLT_REGISTRATION = unsafe { MaybeUninit::zeroed().assume_init() };

        registration.Size = core::mem::size_of::<_FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration = self
            .operations
            .map_or_else(|| core::ptr::null(), |c| c.get().as_ptr());

        registration.ContextRegistration = self
            .context
            .map_or_else(|| core::ptr::null(), |c| c.get().as_ptr());

        registration.FilterUnloadCallback = self.unload_cb;

        Ok(FltRegistration::new(registration))
    }
}

impl FltRegistration {
    fn new(registration: _FLT_REGISTRATION) -> Self {
        Self(registration)
    }

    pub fn register_filter(&self, driver: *mut DRIVER_OBJECT) -> anyhow::Result<FltFilter> {
        let mut filter = core::ptr::null_mut();
        let status = unsafe { FltRegisterFilter(driver as _, &self.0, &mut filter) };

        if !wdk::nt_success(status) {
            Err(anyhow::Error::msg("Failed to register filter"))
        } else {
            Ok(FltFilter::new(NonNull::new(filter).unwrap()))
        }
    }
}
