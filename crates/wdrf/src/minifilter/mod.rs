use core::mem::MaybeUninit;

use wdk_sys::{
    fltmgr::{FltRegisterFilter, FLT_REGISTRATION_VERSION, _FLT_REGISTRATION},
    DRIVER_OBJECT,
};

struct MinifilterRegistrationBuilder {}

pub struct MinifilterBuilder {
    driver: &'static mut DRIVER_OBJECT,
    flags: u32,
}

impl MinifilterBuilder {
    pub fn new(driver: &'static mut DRIVER_OBJECT) -> Self {
        Self { driver, flags: 0 }
    }

    pub fn build(self) {
        let registration: _FLT_REGISTRATION = unsafe { MaybeUninit::zeroed().assume_init() };
        registration.Size = core::mem::size_of::<_FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        unsafe { FltRegisterFilter(self.Driver, Registration, RetFilter) }
    }
}
