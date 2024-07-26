use wdk_sys::{
    fltmgr::{FltBuildDefaultSecurityDescriptor, FltFreeSecurityDescriptor, FLT_PORT_ALL_ACCESS},
    PSECURITY_DESCRIPTOR,
};

pub struct FltSecurityDescriptor(PSECURITY_DESCRIPTOR);

impl FltSecurityDescriptor {
    pub fn try_default_flt() -> anyhow::Result<FltSecurityDescriptor> {
        let mut sd = core::ptr::null_mut();
        unsafe {
            let status = FltBuildDefaultSecurityDescriptor(&mut sd, FLT_PORT_ALL_ACCESS);
            anyhow::ensure!(
                wdk::nt_success(status),
                "Failed to build default security descriptor"
            );
        };

        Ok(FltSecurityDescriptor(sd))
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
