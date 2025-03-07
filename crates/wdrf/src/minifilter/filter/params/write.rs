use core::ffi::c_void;

use wdrf_std::aligned::AsAligned;
use wdrf_std::slice::slice_from_raw_parts_mut_or_empty;
use windows_sys::Wdk::Foundation::MDL;
use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS;

#[repr(C)]
#[allow(non_snake_case)]
struct FltWriteParameter {
    pub Length: u32,
    pub Key: AsAligned<u32>, //aligned 8
    pub ByteOffset: i64,
    pub WriteBuffer: *mut c_void,
    pub MdlAddress: *mut MDL,
}

#[repr(transparent)]
pub struct FltWriteFileRequest<'a> {
    write: &'a FltWriteParameter,
}

impl<'a> FltWriteFileRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let write_ptr = &params.Write as *const _ as *const FltWriteParameter;
            Self { write: &*write_ptr }
        }
    }

    pub fn len(&self) -> u32 {
        self.write.Length
    }

    pub fn offset(&self) -> i64 {
        self.write.ByteOffset
    }

    pub fn user_write_buffer(&self) -> Option<&mut [u8]> {
        unsafe {
            if self.write.WriteBuffer.is_null() {
                None
            } else {
                Some(slice_from_raw_parts_mut_or_empty(
                    self.write.WriteBuffer as _,
                    self.write.Length as _,
                ))
            }
        }
    }
}
