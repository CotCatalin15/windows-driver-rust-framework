use wdrf_std::{NtResult, NtResultEx};
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FltBuildDefaultSecurityDescriptor, FltFreeSecurityDescriptor,
    },
    Win32::Security::PSECURITY_DESCRIPTOR,
};

use crate::minifilter::structs::FLT_PORT_ALL_ACCESS;

pub struct FltSecurityDescriptor(PSECURITY_DESCRIPTOR);

impl FltSecurityDescriptor {
    pub fn try_default_flt() -> NtResult<Self> {
        let mut sd = core::ptr::null_mut();
        unsafe {
            let status = FltBuildDefaultSecurityDescriptor(&mut sd, FLT_PORT_ALL_ACCESS);
            NtResult::from_status(status, || Self(sd))
        }
    }

    pub fn as_ptr(&self) -> PSECURITY_DESCRIPTOR {
        self.0
    }
}

impl Drop for FltSecurityDescriptor {
    fn drop(&mut self) {
        unsafe {
            FltFreeSecurityDescriptor(self.0);
        }
    }
}

impl AsRef<PSECURITY_DESCRIPTOR> for FltSecurityDescriptor {
    fn as_ref(&self) -> &PSECURITY_DESCRIPTOR {
        &self.0
    }
}
