use core::ptr::NonNull;

use wdk_sys::{fltmgr::_FLT_FILTER, DRIVER_OBJECT};
use wdrf_std::string::ntunicode::NtUnicode;

pub trait FrameworkContext {}

pub struct FrameworkBuilder {
    pub(super) driver: NonNull<DRIVER_OBJECT>,
    pub(super) registry: NtUnicode<'static>,
    pub(super) context: Option<&'static mut dyn FrameworkContext>,
    unload_fnc: Option<FrameworkUnloadCallback>,
}

pub type FrameworkUnloadCallback = fn(&mut dyn FrameworkContext);

impl FrameworkBuilder {
    pub fn new(driver: NonNull<DRIVER_OBJECT>, registry: NtUnicode<'static>) -> Self {
        Self {
            driver,
            registry,
            unload_fnc: None,
            context: None,
        }
    }

    pub fn context(&mut self, context: &'static mut dyn FrameworkContext) -> &mut Self {
        self.context = Some(context);
        self
    }

    pub fn unload(&mut self, unload_fnc: FrameworkUnloadCallback) -> &mut Self {
        self.unload_fnc = Some(unload_fnc);
        self
    }

    pub fn build(self) -> DriverFramework {
        DriverFramework {
            driver: self.driver,
            registry: self.registry,
        }
    }
}

pub struct DriverFramework {
    pub driver: NonNull<DRIVER_OBJECT>,
    pub registry: NtUnicode<'static>,
}
