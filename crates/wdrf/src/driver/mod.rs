use core::{alloc::Layout, ffi::c_void, ptr::NonNull};

use anyhow::Error;
use wdk_sys::{
    ntddk::{ExAllocatePool2, ExFreePoolWithTag},
    DRIVER_OBJECT, POOL_FLAG_NON_PAGED, UNICODE_STRING, _DRIVER_OBJECT,
};

pub type DriverUnloadFnc = fn(&mut DriverObject);

pub struct DriverDispatch {
    object: *mut DRIVER_OBJECT,
    context: Option<NonNull<[u8]>>,

    unload: Option<DriverUnloadFnc>,
}

#[allow(dead_code)]
pub struct DriverObject {
    object: *mut DRIVER_OBJECT,
    registry: *const UNICODE_STRING,
    dispatch: DriverDispatch,
}

unsafe impl Sync for DriverObject {}

const DRIVER_CONTEXT_TAG: u32 = 0x1234;
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
    pub fn get_context<C: Sized>(&self) -> Option<&mut C> {
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
    pub fn set_context<C: Sized>(&mut self, value: C) -> anyhow::Result<()> {
        let size = Layout::new::<C>().size() as u64;
        let ptr =
            unsafe { ExAllocatePool2(POOL_FLAG_NON_PAGED, size, DRIVER_CONTEXT_TAG).cast::<u8>() };
        if ptr.is_null() {
            Err(Error::msg("Failed to allocate memory"))
        } else {
            let context = ptr.cast::<C>();
            unsafe {
                *context = value;
            }

            unsafe {
                let context =
                    NonNull::new(core::slice::from_raw_parts_mut(ptr, size as usize)).unwrap();
                self.context = Some(context);
            }
            Ok(())
        }
    }

    #[inline]
    pub fn get_context<C: Sized>(&self) -> Option<&mut C> {
        match &self.context {
            None => None,
            Some(ptr) => unsafe {
                let ptr = &mut *ptr.as_ptr().cast::<C>();
                Some(ptr)
            },
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

    if let Some(fnc) = object.dispatch.unload {
        fnc(object);
    }

    if let Some(ref context) = object.dispatch.context {
        unsafe {
            ExFreePoolWithTag(context.as_ptr().cast::<c_void>(), DRIVER_CONTEXT_TAG);
        }
    }
}
