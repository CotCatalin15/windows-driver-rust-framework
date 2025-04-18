use wdrf_std::aligned::AsAligned;
use windows_sys::Wdk::{
    Foundation::IO_SECURITY_CONTEXT, Storage::FileSystem::Minifilters::FLT_PARAMETERS,
    System::SystemServices::NAMED_PIPE_CREATE_PARAMETERS,
};

#[repr(C)]
#[allow(non_snake_case)]
struct FltCreatePipeParameter {
    pub SecurityContext: *mut IO_SECURITY_CONTEXT,
    pub Options: u32,
    pub Reserved: AsAligned<u16>,
    pub ShareAccess: u16,
    pub Parameters: *mut NAMED_PIPE_CREATE_PARAMETERS,
}

pub struct FltCreatePipeRequest<'a> {
    param: &'a FltCreatePipeParameter,
}

impl<'a> FltCreatePipeRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let pipe_ptr = &params.CreatePipe as *const _ as *const FltCreatePipeParameter;
            Self { param: &*pipe_ptr }
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

    pub fn parameters(&self) -> &'a NAMED_PIPE_CREATE_PARAMETERS {
        unsafe { &*self.param.Parameters }
    }
}
