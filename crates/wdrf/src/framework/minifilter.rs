use core::{any::Any, ptr::NonNull};

use wdk_sys::{
    fltmgr::{
        FltRegisterFilter, FltStartFiltering, FltUnregisterFilter, FLT_CONTEXT_END,
        FLT_CONTEXT_REGISTRATION, FLT_FILTER_UNLOAD_FLAGS, FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
        FLT_OPERATION_REGISTRATION, FLT_REGISTRATION, FLT_REGISTRATION_VERSION,
        PCFLT_RELATED_OBJECTS, _FLT_FILTER,
    },
    DRIVER_OBJECT, NTSTATUS, NT_SUCCESS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::{boxed::Box, string::ntunicode::NtUnicode};

use super::builder::{DriverFramework, FrameworkBuilder};

pub const IRP_MJ_OPERATION_END: u8 = 0x80;
pub type FilterUnload = fn(&mut MinifilterFramework) -> anyhow::Result<()>;

pub struct MinifilterFrameworkBuilder {
    inner_builder: FrameworkBuilder,
    registration: FltRegistrationBuilder,
    unload: Option<FilterUnload>,
}

impl MinifilterFrameworkBuilder {
    pub fn new(driver: NonNull<DRIVER_OBJECT>, registry: NtUnicode<'static>) -> Self {
        Self {
            inner_builder: FrameworkBuilder::new(driver, registry),
            registration: FltRegistrationBuilder::new(),
            unload: None,
        }
    }

    pub fn start_builder<F>(
        driver: NonNull<DRIVER_OBJECT>,
        registry: NtUnicode<'static>,
        main: F,
    ) -> NTSTATUS
    where
        F: FnOnce(&mut MinifilterFrameworkBuilder) -> anyhow::Result<()>,
    {
        let mut builder = MinifilterFrameworkBuilder::new(driver, registry);
        let result = main(&mut builder);
        match result {
            Ok(_) => STATUS_SUCCESS,
            Err(_) => {
                unsafe {
                    core::mem::drop(GLOBAL_MINIFILTER_FRAMEWORK.take());
                }
                STATUS_UNSUCCESSFUL
            }
        }
    }

    pub fn context(&mut self, context: Box<dyn Any>) -> &mut Self {
        self.inner_builder.context(context);
        self
    }

    pub fn unload(&mut self, unload_fnc: FilterUnload) -> &mut Self {
        self.unload = Some(unload_fnc);
        self.registration.unload(unload_fnc);
        self
    }

    pub fn registration<F>(&mut self, builder: F) -> &mut Self
    where
        F: FnOnce(&mut FltRegistrationBuilder),
    {
        builder(&mut self.registration);
        self
    }

    pub fn build(&mut self) -> anyhow::Result<&'static mut MinifilterFramework> {
        let mut filter = core::ptr::null_mut();
        unsafe {
            let mut registration = self.registration.build()?;

            let status = FltRegisterFilter(
                self.inner_builder.driver.as_ptr() as _,
                &mut registration,
                &mut filter,
            );

            anyhow::ensure!(NT_SUCCESS(status), "Failed to register filter");
            anyhow::ensure!(
                !filter.is_null(),
                "FltREgisterFilter returned a null filter"
            );
            let flt_filter = NonNull::new(filter).unwrap();

            GLOBAL_MINIFILTER_FRAMEWORK = Some(MinifilterFramework {
                framework: self.inner_builder.build(),
                unload: self.unload,
                flt_filter: flt_filter.clone(),
            });

            Ok(GLOBAL_MINIFILTER_FRAMEWORK.as_mut().unwrap())
        }
    }
}

pub struct MinifilterFramework {
    #[allow(dead_code)]
    framework: &'static mut DriverFramework,
    unload: Option<FilterUnload>,
    flt_filter: NonNull<_FLT_FILTER>,
}

impl MinifilterFramework {
    pub fn get() -> &'static mut Self {
        unsafe { GLOBAL_MINIFILTER_FRAMEWORK.as_mut().unwrap() }
    }

    pub fn context<C>() -> Option<&'static mut C> {
        unsafe {
            GLOBAL_MINIFILTER_FRAMEWORK
                .as_mut()?
                .framework
                .context
                .as_mut()?
                .downcast_mut()
        }
    }

    pub fn start_filtering(&mut self) -> anyhow::Result<()> {
        unsafe {
            let status = FltStartFiltering(self.flt_filter.as_ptr());
            if NT_SUCCESS(status) {
                Ok(())
            } else {
                Err(anyhow::Error::msg("Failed to start filtering"))
            }
        }
    }

    pub fn filter(&self) -> NonNull<_FLT_FILTER> {
        self.flt_filter
    }
}

impl Drop for MinifilterFramework {
    fn drop(&mut self) {
        unsafe {
            FltUnregisterFilter(self.flt_filter.as_ptr());
        }
    }
}

#[derive(Default)]
pub struct FltRegistrationBuilder {
    flags: u32,
    unload: Option<FilterUnload>,
    context: Option<&'static [FLT_CONTEXT_REGISTRATION]>,
    callbacks: Option<&'static [FLT_OPERATION_REGISTRATION]>,
}

impl FltRegistrationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn unload(&mut self, unload_fnc: FilterUnload) -> &mut Self {
        self.unload = Some(unload_fnc);
        self
    }

    pub fn flag(&mut self, flag: u32) -> &mut Self {
        self.flags = flag;
        self
    }

    pub fn context_registration(
        &mut self,
        context: &'static [FLT_CONTEXT_REGISTRATION],
    ) -> &mut Self {
        self.context = Some(context);
        self
    }

    pub fn callbacks(&mut self, callbacks: &'static [FLT_OPERATION_REGISTRATION]) -> &mut Self {
        self.callbacks = Some(callbacks);
        self
    }

    pub fn build(&mut self) -> anyhow::Result<FLT_REGISTRATION> {
        unsafe {
            if let Some(context) = self.context.as_ref().and_then(|ctx| ctx.last()) {
                anyhow::ensure!(
                    context.ContextType != FLT_CONTEXT_END as _,
                    "Last context value is not FLT_CONTEXT_END"
                );
            }

            if let Some(operation) = self.callbacks.as_ref().and_then(|ctx| ctx.last()) {
                anyhow::ensure!(
                    operation.MajorFunction != IRP_MJ_OPERATION_END,
                    "Last context value is not FLT_CONTEXT_END"
                );
            }

            Ok(FLT_REGISTRATION {
                Size: core::mem::size_of::<FLT_REGISTRATION>() as _,
                Version: FLT_REGISTRATION_VERSION as _,
                Flags: self.flags,
                ContextRegistration: self
                    .context
                    .map_or_else(|| core::ptr::null(), |ctx| ctx.as_ptr()),
                OperationRegistration: self
                    .callbacks
                    .map_or_else(|| core::ptr::null() as _, |op| op.as_ptr()),
                FilterUnloadCallback: self.unload.map(|_| minifilter_unload_callback as _),
                InstanceQueryTeardownCallback: Some(minifilter_instance_teardown),
                ..core::mem::zeroed::<FLT_REGISTRATION>()
            })
        }
    }
}

static mut GLOBAL_MINIFILTER_FRAMEWORK: Option<MinifilterFramework> = None;

/*
NTSTATUS
NullQueryTeardown (
    _In_ PCFLT_RELATED_OBJECTS FltObjects,
    _In_ FLT_INSTANCE_QUERY_TEARDOWN_FLAGS Flags
    )
 */

unsafe extern "C" fn minifilter_instance_teardown(
    _objects: PCFLT_RELATED_OBJECTS,
    _flags: FLT_INSTANCE_QUERY_TEARDOWN_FLAGS,
) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe extern "C" fn minifilter_unload_callback(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    if let Some(mut framework) = GLOBAL_MINIFILTER_FRAMEWORK.take() {
        if let Some(cb) = framework.unload {
            match cb(&mut framework) {
                Ok(_) => {
                    return STATUS_SUCCESS;
                }
                Err(_) => {
                    return STATUS_UNSUCCESSFUL;
                }
            }
        }
        //This is where framework gets dopped
        //drop(framework)
    }

    STATUS_SUCCESS
}
