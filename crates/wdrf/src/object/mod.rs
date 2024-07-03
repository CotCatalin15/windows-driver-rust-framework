use wdk::nt_success;
use wdk_sys::{
    fltmgr::{FltBuildDefaultSecurityDescriptor, FltFreeSecurityDescriptor, FLT_PORT_ALL_ACCESS},
    OBJECT_ATTRIBUTES, PSECURITY_DESCRIPTOR,
};
use wdrf_std::string::ntunicode::NtUnicode;

pub struct SecurityDescriptor(PSECURITY_DESCRIPTOR);

impl SecurityDescriptor {
    pub fn try_default_flt() -> anyhow::Result<SecurityDescriptor> {
        let mut sd = core::ptr::null_mut();
        unsafe {
            let status = FltBuildDefaultSecurityDescriptor(&mut sd, FLT_PORT_ALL_ACCESS);
            anyhow::ensure!(
                nt_success(status),
                "Failed to build default security descriptor"
            );
        };

        Ok(SecurityDescriptor(sd))
    }
}

impl Drop for SecurityDescriptor {
    fn drop(&mut self) {
        unsafe {
            FltFreeSecurityDescriptor(self.0);
        }
    }
}

pub struct ObjectAttribs<'a> {
    _sd: &'a SecurityDescriptor,
    attribs: OBJECT_ATTRIBUTES,
}

impl<'a> ObjectAttribs<'a> {
    pub fn new(
        name: &'a mut NtUnicode<'static>,
        attrib_flags: u32,
        descriptor: &'a SecurityDescriptor,
    ) -> ObjectAttribs<'a> {
        panic!("This aint ok");

        let obj_attribs = OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as _,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: core::ptr::null_mut(),
            Attributes: attrib_flags,
            SecurityDescriptor: descriptor.0,
            SecurityQualityOfService: core::ptr::null_mut(),
        };

        ObjectAttribs {
            _sd: descriptor,
            attribs: obj_attribs,
        }
    }

    pub unsafe fn as_ptr(&self) -> *const OBJECT_ATTRIBUTES {
        &self.attribs
    }

    pub unsafe fn as_ptr_mut(&mut self) -> *mut OBJECT_ATTRIBUTES {
        &mut self.attribs
    }
}
