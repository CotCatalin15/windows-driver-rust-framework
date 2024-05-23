use core::{any::Any, ptr::NonNull};

use wdk_sys::DRIVER_OBJECT;
use wdrf_std::{boxed::Box, string::ntunicode::NtUnicode};

pub struct FrameworkBuilder {
    pub(super) driver: NonNull<DRIVER_OBJECT>,
    pub(super) registry: NtUnicode<'static>,
    pub(super) context: Option<Box<dyn Any>>,
    unload_fnc: Option<FrameworkUnloadCallback>,
}

pub type FrameworkUnloadCallback = fn(&mut DriverFramework);

impl FrameworkBuilder {
    pub fn new(driver: NonNull<DRIVER_OBJECT>, registry: NtUnicode<'static>) -> Self {
        Self {
            driver,
            registry,
            unload_fnc: None,
            context: None,
        }
    }

    //The context here will be droped when the unload callback will be called
    pub fn context(&mut self, context: Box<dyn Any>) -> &mut Self {
        self.context = Some(context);
        self
    }

    pub fn unload(mut self, unload_fnc: FrameworkUnloadCallback) -> Self {
        self.unload_fnc = Some(unload_fnc);
        self
    }

    pub fn build(&mut self) -> &'static mut DriverFramework {
        unsafe {
            GLOBAL_DRIVER_FRAMEWORK = Some(DriverFramework {
                driver: self.driver,
                registry: self.registry,
                unload_fnc: self.unload_fnc,
                context: self.context.take(),
            });

            (*self.driver.as_ptr()).DriverUnload =
                self.unload_fnc.map(|_| framework_driver_unload as _);

            GLOBAL_DRIVER_FRAMEWORK.as_mut().unwrap()
        }
    }
}

pub struct DriverFramework {
    pub driver: NonNull<DRIVER_OBJECT>,
    pub registry: NtUnicode<'static>,
    pub(super) unload_fnc: Option<FrameworkUnloadCallback>,
    pub(super) context: Option<Box<dyn Any>>,
}

impl DriverFramework {
    pub fn get() -> &'static mut DriverFramework {
        unsafe { GLOBAL_DRIVER_FRAMEWORK.as_mut().unwrap() }
    }

    pub fn context<C>() -> Option<&'static C> {
        unsafe {
            GLOBAL_DRIVER_FRAMEWORK
                .as_ref()?
                .context
                .as_ref()?
                .downcast_ref()
        }
    }

    pub unsafe fn context_mut<C>() -> Option<&'static mut C> {
        unsafe {
            GLOBAL_DRIVER_FRAMEWORK
                .as_mut()?
                .context
                .as_mut()?
                .downcast_mut()
        }
    }
}

static mut GLOBAL_DRIVER_FRAMEWORK: Option<DriverFramework> = None;

pub unsafe extern "C" fn framework_driver_unload(_driver: *mut DRIVER_OBJECT) {
    if let Some(mut framework) = GLOBAL_DRIVER_FRAMEWORK.take() {
        framework.unload_fnc.inspect(|cb| cb(&mut framework));

        // This is where the drop occurs => drops the context
        // drop(framework);
    }
}
