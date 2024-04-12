use core::{any::Any, cell::UnsafeCell};

use wdk_sys::{DRIVER_OBJECT, UNICODE_STRING, _DRIVER_OBJECT};
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
    object: *mut DRIVER_OBJECT,
    registry: *const UNICODE_STRING,
    dispatch: DriverDispatch,
}

unsafe impl Sync for DriverObject {}

static mut GLOBAL_DRIVER: DriverObject = DriverObject::zeroed();

impl DriverObject {
    pub fn init(
        driver: *mut DRIVER_OBJECT,
        registry: *const UNICODE_STRING,
    ) -> Option<(&'static mut Self, &'static mut DriverDispatch)> {
        if driver.is_null() || registry.is_null() {
            None
        } else {
            #[allow(static_mut_refs)]
            unsafe {
                GLOBAL_DRIVER = Self {
                    object: driver,
                    registry,
                    dispatch: DriverDispatch::new(driver),
                };

                Some((&mut GLOBAL_DRIVER, &mut GLOBAL_DRIVER.dispatch))
            }
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

    if let Some(context) = object.dispatch.context.take() {
        core::mem::drop(context);
    }
}
