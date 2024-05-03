use core::{any::Any, ptr::NonNull};

use wdk_sys::{
    fltmgr::{
        FltRegisterFilter, FltUnregisterFilter, FLT_CONTEXT_END, FLT_CONTEXT_REGISTRATION,
        FLT_FILTER_UNLOAD_FLAGS, FLT_OPERATION_REGISTRATION, FLT_REGISTRATION,
        FLT_REGISTRATION_VERSION, _FLT_FILTER,
    },
    DRIVER_OBJECT, NTSTATUS, NT_SUCCESS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::string::ntunicode::NtUnicode;

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

    pub fn context(&mut self, context: &'static mut dyn Any) -> &mut Self {
        self.inner_builder.context(context);
        self
    }

    pub fn unload(&mut self, unload_fnc: FilterUnload) -> &mut Self {
        self.unload = Some(unload_fnc);
        self.registration.unload(unload_fnc);
        self
    }

    pub fn build(self) -> anyhow::Result<MinifilterFramework> {
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
        }
        let flt_filter = NonNull::new(filter).unwrap();

        Ok(MinifilterFramework {
            framework: self.inner_builder.build(),
            unload: self.unload,
            flt_filter: flt_filter,
        })
    }
}

pub struct MinifilterFramework {
    framework: &'static mut DriverFramework,
    unload: Option<FilterUnload>,
    flt_filter: NonNull<_FLT_FILTER>,
}

impl MinifilterFramework {
    pub fn get() -> &'static mut Self {
        unsafe { GLOBAL_MINIFILTER_FRAMEWORK.as_mut().unwrap() }
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
    context: &'static [FLT_CONTEXT_REGISTRATION],
    callbacks: &'static [FLT_OPERATION_REGISTRATION],
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
        self.context = context;
        self
    }

    pub fn callbacks(&mut self, callbacks: &'static [FLT_OPERATION_REGISTRATION]) -> &mut Self {
        self.callbacks = callbacks;
        self
    }

    pub fn build(self) -> anyhow::Result<FLT_REGISTRATION> {
        unsafe {
            if let Some(context) = self.context.last() {
                anyhow::ensure!(
                    context.ContextType != FLT_CONTEXT_END as _,
                    "Last context value is not FLT_CONTEXT_END"
                );
            }

            if let Some(operation) = self.callbacks.last() {
                anyhow::ensure!(
                    operation.MajorFunction != IRP_MJ_OPERATION_END,
                    "Last context value is not FLT_CONTEXT_END"
                );
            }

            Ok(FLT_REGISTRATION {
                Size: core::mem::size_of::<FLT_REGISTRATION>() as _,
                Version: FLT_REGISTRATION_VERSION as _,
                Flags: self.flags,
                ContextRegistration: self.context.as_ptr(),
                OperationRegistration: self.callbacks.as_ptr(),
                FilterUnloadCallback: self.unload.map(|_| minifilter_unload_callback as _),
                ..core::mem::zeroed::<FLT_REGISTRATION>()
            })
        }
    }
}

static mut GLOBAL_MINIFILTER_FRAMEWORK: Option<MinifilterFramework> = None;

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
