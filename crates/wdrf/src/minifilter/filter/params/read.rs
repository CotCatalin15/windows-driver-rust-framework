use core::ffi::c_void;

use wdrf_std::aligned::AsAligned;
use wdrf_std::slice::slice_from_raw_parts_mut_or_empty;
use windows_sys::Wdk::Foundation::MDL;
use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS;

#[repr(C)]
#[allow(non_snake_case)]
struct FltReadParameter {
    pub Length: u32,
    pub Key: AsAligned<u32>, //aligned 8
    pub ByteOffset: i64,
    pub ReadBuffer: *mut c_void,
    pub MdlAddress: *mut MDL,
}

pub struct FltReadFileRequest<'a> {
    read: &'a FltReadParameter,
}

impl<'a> FltReadFileRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let read_ptr = &params.Read as *const _ as *const FltReadParameter;
            Self { read: &*read_ptr }
        }
    }

    pub fn len(&self) -> u32 {
        self.read.Length
    }

    pub fn user_read_buffer(&self) -> Option<&mut [u8]> {
        unsafe {
            if self.read.ReadBuffer.is_null() {
                None
            } else {
                Some(slice_from_raw_parts_mut_or_empty(
                    self.read.ReadBuffer as _,
                    self.read.Length as _,
                ))
            }
        }
    }
}
