#![no_std]
#![feature(sync_unsafe_cell)]

pub mod context;
pub mod driver;
pub mod minifilter;
pub mod object;
pub mod process;

use driver::{DriverDispatch, DriverObject};
use wdk_sys::{
    DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

pub struct FrameworkContext;

pub struct Framework {}

impl Framework {
    pub fn run_entry<F>(
        driver: *mut DRIVER_OBJECT,
        registry_path: *const UNICODE_STRING,
        main_fnc: F,
    ) -> NTSTATUS
    where
        F: 'static + FnOnce(&'static mut DriverObject, &mut DriverDispatch) -> anyhow::Result<()>,
    {
        match DriverObject::init(driver, registry_path, main_fnc) {
            Ok(()) => STATUS_SUCCESS,
            Err(_) => STATUS_UNSUCCESSFUL,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Error;
    use wdk_sys::{DRIVER_OBJECT, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING};

    use crate::Framework;

    extern crate std;

    #[test]
    fn test_result() {
        let driver_obj = 0x1 as *mut DRIVER_OBJECT;
        let registry_path = 0x1 as *mut UNICODE_STRING;

        let mut status =
            Framework::run_entry(driver_obj, registry_path, |_driver, dispatch| Ok(()));
        assert_eq!(status, STATUS_SUCCESS);

        status = Framework::run_entry(driver_obj, registry_path, |_driver, dispatch| {
            Err(Error::msg("Test"))
        });
        assert_eq!(status, STATUS_UNSUCCESSFUL);
    }
}
