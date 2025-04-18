use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS,
    Win32::System::WindowsProgramming::FILE_INFORMATION_CLASS,
};

use wdrf_std::aligned::AsAligned;

#[repr(C)]
#[allow(non_snake_case)]
pub struct FltQueryFileInformationParameter {
    pub Length: u32,
    pub FileInformationClass: AsAligned<FILE_INFORMATION_CLASS>,
    pub InfoBuffer: *mut ::core::ffi::c_void,
}

pub struct FltQueryFileInformationRequest<'a> {
    param: &'a FltQueryFileInformationParameter,
}

impl<'a> FltQueryFileInformationRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let query_ptr =
                &params.QueryFileInformation as *const _ as *const FltQueryFileInformationParameter;
            Self { param: &*query_ptr }
        }
    }

    pub fn length(&self) -> u32 {
        self.param.Length
    }

    pub fn file_information_class(&self) -> FILE_INFORMATION_CLASS {
        self.param.FileInformationClass.ptr
    }

    pub fn info_buffer(&self) -> *mut ::core::ffi::c_void {
        self.param.InfoBuffer
    }
}
