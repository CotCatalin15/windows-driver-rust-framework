use core::any::Any;

use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    NtResult, NtStatusError,
};
use windows_sys::{
    Wdk::{
        Foundation::DRIVER_OBJECT,
        Storage::FileSystem::Minifilters::{
            FLT_REGISTRATION, FLT_REGISTRATION_VERSION, PFLT_FILTER_UNLOAD_CALLBACK,
        },
    },
    Win32::Foundation::{NOERROR, STATUS_NO_MEMORY},
};

use crate::{
    context::ContextRegistry,
    minifilter::filter::{
        flt_op_callbacks::flt_minifilter_unload_implementation,
        framework::{
            MinifilterContext, MinifilterContextAny, MinifilterFramework, GLOBAL_MINIFILTER,
        },
        registration, FilterUnload,
    },
};

use super::IntoFltOpRegistrationFactory;

pub struct MinifilterFrameworkBuilder<I: IntoFltOpRegistrationFactory, C = ()>
where
    C: 'static + Send + Sync,
{
    flags: u32,
    op_factory: I,
    minifilter_context: C,
    unload: PFLT_FILTER_UNLOAD_CALLBACK,
}

impl<I> MinifilterFrameworkBuilder<I, ()>
where
    I: IntoFltOpRegistrationFactory<MinifilterContext = ()>,
{
    pub fn new<F>(factory: F) -> Self
    where
        F: FnOnce() -> I,
    {
        let factorty = factory();
        Self {
            flags: 0,
            op_factory: factorty,
            minifilter_context: (),
            unload: None,
        }
    }
}

impl<I, C> MinifilterFrameworkBuilder<I, C>
where
    C: 'static + Send + Sync + TaggedObject + Any,
    I: IntoFltOpRegistrationFactory<MinifilterContext = C>,
{
    pub fn new_with_context<F>(factory: F, context: C) -> Self
    where
        F: FnOnce() -> I,
    {
        let factorty = factory();
        Self {
            flags: 0,
            op_factory: factorty,
            minifilter_context: context,
            unload: None,
        }
    }

    pub fn unload<F: FilterUnload<MinifilterContext = C>>(mut self, _unload: F) -> Self {
        self.unload = Some(flt_minifilter_unload_implementation::<F>);
        self
    }

    pub fn build_and_register<R: ContextRegistry>(
        mut self,
        r: &'static R,
        driver: *const DRIVER_OBJECT,
    ) -> NtResult<()> {
        let registration = self.create_registratrion();

        let context = MinifilterContext::try_create(self.minifilter_context)?;

        let mut framework = MinifilterFramework::new(context.into_any());

        unsafe {
            framework.register_filter(driver, registration)?;
        }
        GLOBAL_MINIFILTER.init(r, || framework).unwrap();

        Ok(())
    }

    fn create_registratrion(&mut self) -> FLT_REGISTRATION {
        let registration_operations = self.op_factory.into_operations();

        let mut registration: FLT_REGISTRATION = unsafe { core::mem::zeroed() };
        registration.Size = core::mem::size_of::<FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration = if registration_operations.len() == 0 {
            core::ptr::null()
        } else {
            registration_operations.as_ptr()
        };
        registration.ContextRegistration = core::ptr::null();
        registration.FilterUnloadCallback = self.unload;
        //registration.InstanceSetupCallback = self.instance_setup;
        /*
        registration.InstanceQueryTeardownCallback = self.instance_query_teardown;
        registration.InstanceTeardownCompleteCallback = self.instance_teardown;
        registration.InstanceTeardownStartCallback = self.instance_teardown_start;
        registration.GenerateFileNameCallback = self.generate_filename;
        registration.NormalizeNameComponentCallback = self.normalize_name;
        registration.NormalizeContextCleanupCallback = self.normalize_context;
        registration.TransactionNotificationCallback = self.transcation_notification;
        registration.NormalizeNameComponentExCallback = self.normalize_name_ex;
        registration.SectionNotificationCallback = self.section_notification;
        */

        registration
    }
}
