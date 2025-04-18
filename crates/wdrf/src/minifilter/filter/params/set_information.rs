use wdrf_std::aligned::AsAligned;
use windows_sys::{
    Wdk::{
        Foundation::FILE_OBJECT,
        Storage::FileSystem::Minifilters::{FLT_PARAMETERS, FLT_PARAMETERS_27_0},
    },
    Win32::{Foundation::HANDLE, System::WindowsProgramming::FILE_INFORMATION_CLASS},
};

#[repr(C)]
#[allow(non_snake_case)]
pub struct FltSetFileInformationParameter {
    pub Length: u32,
    pub FileInformationClass: AsAligned<FILE_INFORMATION_CLASS>,
    pub ParentOfTarget: *mut FILE_OBJECT,
    pub Anonymous: FLT_PARAMETERS_27_0,
    pub InfoBuffer: *mut ::core::ffi::c_void,
}

pub struct FltSetFileInformationRequest<'a> {
    param: &'a FltSetFileInformationParameter,
}

impl<'a> FltSetFileInformationRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let query_ptr =
                &params.SetFileInformation as *const _ as *const FltSetFileInformationParameter;
            Self { param: &*query_ptr }
        }
    }

    pub fn length(&self) -> u32 {
        self.param.Length
    }

    pub fn file_information_class(&self) -> FILE_INFORMATION_CLASS {
        self.param.FileInformationClass.get()
    }

    pub fn parent_of_target(&self) -> Option<&'a FILE_OBJECT> {
        unsafe { self.param.ParentOfTarget.as_ref() }
    }

    pub fn info_buffer(&self) -> *mut ::core::ffi::c_void {
        self.param.InfoBuffer
    }
}
