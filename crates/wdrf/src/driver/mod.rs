use core::{any::Any, cell::UnsafeCell};

use wdk_sys::{DRIVER_OBJECT, IRP, UNICODE_STRING, _DRIVER_OBJECT};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
};

pub type DriverUnloadFnc = fn(&mut DriverObject);

pub struct DriverDispatch {
    object: *mut DRIVER_OBJECT,
    context: Option<UnsafeCell<Box<dyn Any>>>,

    unload: Option<DriverUnloadFnc>,
}

#[allow(dead_code)]
pub struct DriverObject {
    pub object: *mut DRIVER_OBJECT,
    registry: *const UNICODE_STRING,
    dispatch: DriverDispatch,
}

unsafe impl Sync for DriverObject {}

static mut GLOBAL_DRIVER: DriverObject = DriverObject::zeroed();

impl DriverObject {
    pub fn init<F>(
        driver: *mut DRIVER_OBJECT,
        registry: *const UNICODE_STRING,
        main_fnc: F,
    ) -> anyhow::Result<()>
    where
        F: 'static + FnOnce(&'static mut DriverObject, &mut DriverDispatch) -> anyhow::Result<()>,
    {
        if driver.is_null() || registry.is_null() {
            Err(anyhow::Error::msg("Invalid entry parameters"))
        } else {
            #[allow(static_mut_refs)]
            unsafe {
                GLOBAL_DRIVER = Self {
                    object: driver,
                    registry,
                    dispatch: DriverDispatch::new(driver),
                };

                match main_fnc(&mut GLOBAL_DRIVER, &mut GLOBAL_DRIVER.dispatch) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        Self::uninit();
                        Err(e)
                    }
                }
            }
        }
    }

    ///
    /// # Safety
    /// MUST be called in 2 places:
    /// 1. On driver init fail
    /// 2. On driver unload
    /// Calling it from other places will result in the context becoming invalid
    ///
    unsafe fn uninit() {
        let object = DriverObject::get_global();

        if let Some(context) = object.dispatch.context.take() {
            core::mem::drop(context);
        }
    }

    #[inline]
    pub fn get_global() -> &'static mut DriverObject {
        #[allow(static_mut_refs)]
        unsafe {
            &mut GLOBAL_DRIVER
        }
    }

    #[inline]
    pub fn get_context<C: Sized>(&self) -> Option<&'static mut C> {
        self.dispatch.get_context()
    }

    const fn zeroed() -> Self {
        unsafe { core::mem::zeroed::<DriverObject>() }
    }
}

impl DriverDispatch {
    fn new(object: *mut DRIVER_OBJECT) -> Self {
        Self {
            object,
            context: Default::default(),
            unload: Default::default(),
        }
    }

    #[inline]
    pub fn set_context<C: TaggedObject + 'static>(&mut self, value: C) -> anyhow::Result<()> {
        let context = Box::try_create(value)?;
        self.context = Some(UnsafeCell::new(context));

        Ok(())
    }

    #[inline]
    pub fn get_context<C>(&self) -> Option<&'static mut C> {
        match &self.context {
            None => None,
            Some(context) => unsafe { (*context.get()).downcast_mut::<C>() },
        }
    }

    pub fn set_unload(&mut self, unload_fnc: DriverUnloadFnc) {
        self.unload = Some(unload_fnc);

        unsafe {
            (*self.object).DriverUnload = Some(driver_unload_impl);
        }
    }
}

/*
pub type DRIVER_UNLOAD = ::core::option::Option<
    unsafe extern "C" fn(DriverObject: *mut _DRIVER_OBJECT),
>; */

unsafe extern "C" fn driver_unload_impl(_: *mut _DRIVER_OBJECT) {
    let object = DriverObject::get_global();

    if let Some(ref fnc) = object.dispatch.unload {
        fnc(object);
    }

    DriverObject::uninit();
}
