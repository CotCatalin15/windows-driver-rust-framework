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
        PFLT_GENERATE_FILE_NAME, PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK,
        PFLT_INSTANCE_SETUP_CALLBACK, PFLT_INSTANCE_TEARDOWN_CALLBACK,
        PFLT_NORMALIZE_CONTEXT_CLEANUP, PFLT_NORMALIZE_NAME_COMPONENT,
        PFLT_NORMALIZE_NAME_COMPONENT_EX, PFLT_SECTION_CONFLICT_NOTIFICATION_CALLBACK,
        PFLT_TRANSACTION_NOTIFICATION_CALLBACK,
    },
};

pub struct FltRegistration(FLT_REGISTRATION);

pub struct FltContextRegistrationSlice<const SIZE: usize> {
    inner: [FLT_CONTEXT_REGISTRATION; SIZE],
}

pub struct FltOperationRegistrationSlice<const SIZE: usize> {
    inner: [FLT_OPERATION_REGISTRATION; SIZE],
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

impl<const SIZE: usize> FltContextRegistrationSlice<SIZE> {
    pub const fn new(context: [FLT_CONTEXT_REGISTRATION; SIZE]) -> Option<Self> {
        if let Some(last) = context.last() {
            if last.ContextType == FLT_CONTEXT_END as _ {
                return Some(Self { inner: context });
            }
        }
        None
    }

    pub fn get(&self) -> &[FLT_CONTEXT_REGISTRATION] {
        &self.inner
    }
}

impl<const SIZE: usize> FltOperationRegistrationSlice<SIZE> {
    pub const fn new(context: [FLT_OPERATION_REGISTRATION; SIZE]) -> Option<Self> {
        if let Some(last) = context.last() {
            if last.MajorFunction == IRP_MJ_OPERATION_END as _ {
                return Some(Self { inner: context });
            }
        }
        None
    }

    pub fn get(&self) -> &[FLT_OPERATION_REGISTRATION] {
        &self.inner
    }
}

unsafe impl<const SIZE: usize> Sync for FltOperationRegistrationSlice<SIZE> {}
unsafe impl<const SIZE: usize> Sync for FltContextRegistrationSlice<SIZE> {}

/*
   pub SectionNotificationCallback: PFLT_SECTION_CONFLICT_NOTIFICATION_CALLBACK,
*/

#[derive(Default)]
pub struct FltRegistrationBuilder<'a> {
    flags: u32,
    context: Option<&'a [FLT_CONTEXT_REGISTRATION]>,
    operations: Option<&'a [FLT_OPERATION_REGISTRATION]>,
    unload_cb: PFLT_FILTER_UNLOAD_CALLBACK,
    instance_setup: PFLT_INSTANCE_SETUP_CALLBACK,
    instance_query_teardown: PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK,
    instance_teardown: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    instance_teardown_start: PFLT_INSTANCE_TEARDOWN_CALLBACK,
    generate_filename: PFLT_GENERATE_FILE_NAME,
    normalize_name: PFLT_NORMALIZE_NAME_COMPONENT,
    normalize_context: PFLT_NORMALIZE_CONTEXT_CLEANUP,
    transcation_notification: PFLT_TRANSACTION_NOTIFICATION_CALLBACK,
    normalize_name_ex: PFLT_NORMALIZE_NAME_COMPONENT_EX,
    section_notification: PFLT_SECTION_CONFLICT_NOTIFICATION_CALLBACK,
}

impl<'a> FltRegistrationBuilder<'a> {
    pub fn new() -> Self {
        Self {
            flags: 0,
            context: None,
            operations: None,
            unload_cb: None,
            instance_setup: None,
            instance_query_teardown: None,
            instance_teardown: None,
            instance_teardown_start: None,
            generate_filename: None,
            normalize_name: None,
            normalize_context: None,
            transcation_notification: None,
            normalize_name_ex: None,
            section_notification: None,
        }
    }

    pub fn contexts(mut self, context: &'a [FLT_CONTEXT_REGISTRATION]) -> Self {
        self.context = Some(context);
        self
    }

    pub fn operations(mut self, op: &'a [FLT_OPERATION_REGISTRATION]) -> Self {
        self.operations = Some(op);
        self
    }

    pub fn unload(mut self, unload: PFLT_FILTER_UNLOAD_CALLBACK) -> Self {
        self.unload_cb = unload;
        self
    }

    pub fn instance_setup(mut self, callback: PFLT_INSTANCE_SETUP_CALLBACK) -> Self {
        self.instance_setup = callback;
        self
    }

    pub fn instance_query_teardown(
        mut self,
        callback: PFLT_INSTANCE_QUERY_TEARDOWN_CALLBACK,
    ) -> Self {
        self.instance_query_teardown = callback;
        self
    }

    pub fn instance_teardown_complete(mut self, callback: PFLT_INSTANCE_TEARDOWN_CALLBACK) -> Self {
        self.instance_teardown = callback;
        self
    }

    pub fn instance_teardown_start(mut self, callback: PFLT_INSTANCE_TEARDOWN_CALLBACK) -> Self {
        self.instance_teardown_start = callback;
        self
    }

    pub fn generate_filename(mut self, callback: PFLT_GENERATE_FILE_NAME) -> Self {
        self.generate_filename = callback;
        self
    }

    pub fn normalize_name(mut self, callback: PFLT_NORMALIZE_NAME_COMPONENT) -> Self {
        self.normalize_name = callback;
        self
    }

    pub fn normalize_context(mut self, callback: PFLT_NORMALIZE_CONTEXT_CLEANUP) -> Self {
        self.normalize_context = callback;
        self
    }

    pub fn transcation_notification(
        mut self,
        callback: PFLT_TRANSACTION_NOTIFICATION_CALLBACK,
    ) -> Self {
        self.transcation_notification = callback;
        self
    }

    pub fn normalize_name_ex(mut self, callback: PFLT_NORMALIZE_NAME_COMPONENT_EX) -> Self {
        self.normalize_name_ex = callback;
        self
    }

    pub fn section_notification(
        mut self,
        callback: PFLT_SECTION_CONFLICT_NOTIFICATION_CALLBACK,
    ) -> Self {
        self.section_notification = callback;
        self
    }

    pub fn build(self) -> anyhow::Result<FltRegistration> {
        let mut registration: FLT_REGISTRATION = unsafe { MaybeUninit::zeroed().assume_init() };

        registration.Size = core::mem::size_of::<FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration =
            self.operations.map_or_else(core::ptr::null, |c| c.as_ptr());
        registration.ContextRegistration =
            self.context.map_or_else(core::ptr::null, |c| c.as_ptr());
        registration.FilterUnloadCallback = self.unload_cb;
        registration.InstanceSetupCallback = self.instance_setup;
        registration.InstanceQueryTeardownCallback = self.instance_query_teardown;
        registration.InstanceTeardownCompleteCallback = self.instance_teardown;
        registration.InstanceTeardownStartCallback = self.instance_teardown_start;
        registration.GenerateFileNameCallback = self.generate_filename;
        registration.NormalizeNameComponentCallback = self.normalize_name;
        registration.NormalizeContextCleanupCallback = self.normalize_context;
        registration.TransactionNotificationCallback = self.transcation_notification;
        registration.NormalizeNameComponentExCallback = self.normalize_name_ex;
        registration.SectionNotificationCallback = self.section_notification;

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
