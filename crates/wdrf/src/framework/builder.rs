use core::{any::Any, ptr::NonNull};

use wdk_sys::DRIVER_OBJECT;
use wdrf_std::string::ntunicode::NtUnicode;

pub struct FrameworkBuilder {
    pub(super) driver: NonNull<DRIVER_OBJECT>,
    pub(super) registry: NtUnicode<'static>,
    pub(super) context: Option<&'static mut dyn Any>,
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
    pub fn context(&mut self, context: &'static mut dyn core::any::Any) -> &mut Self {
        self.context = Some(context);
        self
    }

    pub fn unload(&mut self, unload_fnc: FrameworkUnloadCallback) -> &mut Self {
        self.unload_fnc = Some(unload_fnc);
        self
    }

    pub fn build(self) -> &'static mut DriverFramework {
        unsafe {
            GLOBAL_DRIVER_FRAMEWORK = Some(DriverFramework {
                driver: self.driver,
                registry: self.registry,
                context: DriverFrameworkContext {
                    unload_fnc: self.unload_fnc,
                    context: self.context,
                },
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
    context: DriverFrameworkContext,
}

impl DriverFramework {
    pub fn get() -> &'static mut DriverFramework {
        unsafe { GLOBAL_DRIVER_FRAMEWORK.as_mut().unwrap() }
    }
}

struct DriverFrameworkContext {
    pub(super) unload_fnc: Option<FrameworkUnloadCallback>,

    ///
    /// Store the context here so it gets droped when unload is called
    /// Static objects are not automatically destroyed when unload is called
    /// So it has to be done manualy
    ///
    #[allow(dead_code)]
    pub(super) context: Option<&'static mut dyn Any>,
}

unsafe impl Sync for DriverFrameworkContext {}

static mut GLOBAL_DRIVER_FRAMEWORK: Option<DriverFramework> = None;

pub unsafe extern "C" fn framework_driver_unload(_driver: *mut DRIVER_OBJECT) {
    if let Some(mut framework) = GLOBAL_DRIVER_FRAMEWORK.take() {
        framework
            .context
            .unload_fnc
            .inspect(|cb| cb(&mut framework));

        // This is where the drop occurs => drops the context
        // drop(framework);
    }
}
