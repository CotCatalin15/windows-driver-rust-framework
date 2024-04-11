use wdk_sys::fltmgr::{
    FltRegisterFilter, FLT_REGISTRATION, FLT_REGISTRATION_VERSION, _FLT_REGISTRATION,
};

use crate::driver::DriverObject;

struct MiniFilter {}

struct FltRegistration(FLT_REGISTRATION);

impl FltRegistration {
    pub fn new() -> Self {
        let mut registration = unsafe { core::mem::zeroed::<FLT_REGISTRATION>() };

        registration.Size = core::mem::size_of::<FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;

        Self(registration)
    }
}

impl MiniFilter {
    pub fn new(driver: DriverObject) {
        unsafe {}
    }
}

fn test() {}
