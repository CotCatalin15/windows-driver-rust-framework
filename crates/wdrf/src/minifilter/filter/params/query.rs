use wdrf_std::slice::slice_from_raw_parts_mut_or_empty;
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{FLT_PARAMETERS, FLT_PARAMETERS_19},
    Win32::System::WindowsProgramming::FILE_INFORMATION_CLASS,
};

pub struct FltQueryFileRequest<'a> {
    query_params: &'a mut FLT_PARAMETERS_19,
}

impl<'a> FltQueryFileRequest<'a> {
    pub unsafe fn new(params: &'a mut FLT_PARAMETERS) -> Self {
        Self {
            query_params: &mut params.QueryFileInformation,
        }
    }

    pub unsafe fn raw_query(&'a mut self) -> &'a mut FLT_PARAMETERS_19 {
        self.query_params
    }

    pub fn class(&self) -> FILE_INFORMATION_CLASS {
        self.query_params.FileInformationClass
    }

    pub fn buffer(&self) -> &'a mut [u8] {
        unsafe {
            slice_from_raw_parts_mut_or_empty(
                self.query_params.InfoBuffer as _,
                self.query_params.Length as _,
            )
        }
    }
}
