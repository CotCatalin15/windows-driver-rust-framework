use core::ptr::NonNull;

use wdk_sys::{
    fltmgr::{
        FltRegisterFilter, FLT_FILTER_UNLOAD_FLAGS, FLT_REGISTRATION, FLT_REGISTRATION_VERSION,
        _FLT_FILTER,
    },
    DRIVER_OBJECT, NTSTATUS, NT_SUCCESS, STATUS_SUCCESS,
};
use wdrf_std::string::ntunicode::NtUnicode;

use super::builder::{DriverFramework, FrameworkBuilder, FrameworkContext};

pub type FilterUnload = fn(&mut dyn FrameworkContext) -> anyhow::Result<()>;

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

    pub fn context(&mut self, context: &'static mut dyn FrameworkContext) -> &mut Self {
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
            let mut registration = self.registration.build();

            let status = FltRegisterFilter(
                self.inner_builder.driver.as_ptr() as _,
                &mut registration,
                &mut filter,
            );

            anyhow::ensure!(NT_SUCCESS(status), "Failed to register filter");
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
    framework: DriverFramework,
    unload: Option<FilterUnload>,
    flt_filter: NonNull<_FLT_FILTER>,
}

#[derive(Default)]
pub struct FltRegistrationBuilder {
    flags: u32,
    unload: Option<FilterUnload>,
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

    pub fn build(self) -> FLT_REGISTRATION {
        unsafe {
            FLT_REGISTRATION {
                Size: core::mem::size_of::<FLT_REGISTRATION>() as _,
                Version: FLT_REGISTRATION_VERSION as _,
                Flags: self.flags,
                ContextRegistration: todo!(),
                OperationRegistration: todo!(),
                FilterUnloadCallback: self.unload.map(|_| minifilter_unload_callback as _),
                ..core::mem::zeroed::<FLT_REGISTRATION>()
            }
        }
    }
}

unsafe extern "C" fn minifilter_unload_callback(flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    STATUS_SUCCESS
}
