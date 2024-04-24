use core::ptr::NonNull;

use anyhow::bail;
use wdk_sys::{
    fltmgr::{FltRegisterFilter, FLT_REGISTRATION, FLT_REGISTRATION_VERSION, _FLT_FILTER},
    NTSTATUS, NT_SUCCESS, STATUS_SUCCESS,
};

use crate::driver::DriverObject;

type FltUnloadCallback = fn(filter: &mut FltFilter, flags: u32);

pub struct FltRegistration {
    flags: u32,
    unload: Option<FltUnloadCallback>,
}

impl Into<FLT_REGISTRATION> for FltRegistration {
    fn into(self) -> FLT_REGISTRATION {
        FLT_REGISTRATION {
            Size: core::mem::size_of::<FLT_REGISTRATION>() as _,
            Version: FLT_REGISTRATION_VERSION as _,
            Flags: self.flags,
            ContextRegistration: todo!(),
            OperationRegistration: todo!(),
            FilterUnloadCallback: todo!(),
            InstanceSetupCallback: todo!(),
            InstanceQueryTeardownCallback: todo!(),
            InstanceTeardownStartCallback: todo!(),
            InstanceTeardownCompleteCallback: todo!(),
            GenerateFileNameCallback: todo!(),
            NormalizeNameComponentCallback: todo!(),
            NormalizeContextCleanupCallback: todo!(),
            TransactionNotificationCallback: todo!(),
            NormalizeNameComponentExCallback: todo!(),
            SectionNotificationCallback: todo!(),
        }
    }
}

pub struct FltFilter(NonNull<_FLT_FILTER>);

impl FltFilter {
    pub fn new(driver: &mut DriverObject, registration: FltRegistration) -> anyhow::Result<Self> {
        unsafe {
            let mut filter = core::ptr::null_mut();
            let status: NTSTATUS =
                FltRegisterFilter(driver.object as _, &registration.into(), &mut filter);

            if NT_SUCCESS(status) {
                Ok(Self(NonNull::new(filter).unwrap()))
            } else {
                bail!("Failed to create minifilter status: {}", status);
            }
        }
    }
}
