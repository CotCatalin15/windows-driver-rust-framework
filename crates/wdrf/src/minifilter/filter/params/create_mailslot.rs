use wdrf_std::aligned::AsAligned;
use windows_sys::Wdk::{
    Foundation::IO_SECURITY_CONTEXT, Storage::FileSystem::Minifilters::FLT_PARAMETERS,
    System::SystemServices::MAILSLOT_CREATE_PARAMETERS,
};

#[repr(C)]
#[allow(non_snake_case)]
struct FltCreateMailslotParameter {
    pub SecurityContext: *mut IO_SECURITY_CONTEXT,
    pub Options: u32,
    pub Reserved: AsAligned<u16>,
    pub ShareAccess: u16,
    pub Parameters: *mut MAILSLOT_CREATE_PARAMETERS,
}

pub struct FltCreateMailslotRequest<'a> {
    param: &'a FltCreateMailslotParameter,
}

impl<'a> FltCreateMailslotRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let mailslot_ptr =
                &params.CreateMailslot as *const _ as *const FltCreateMailslotParameter;
            Self {
                param: &*mailslot_ptr,
            }
        }
    }

    pub fn security_context(&self) -> Option<&'a IO_SECURITY_CONTEXT> {
        unsafe { self.param.SecurityContext.as_ref() }
    }

    pub fn options(&self) -> u32 {
        self.param.Options
    }

    pub fn reserved(&self) -> u16 {
        self.param.Reserved.ptr
    }

    pub fn share_access(&self) -> u16 {
        self.param.ShareAccess
    }

    pub fn parameters(&self) -> &MAILSLOT_CREATE_PARAMETERS {
        unsafe { &*self.param.Parameters }
    }
}
