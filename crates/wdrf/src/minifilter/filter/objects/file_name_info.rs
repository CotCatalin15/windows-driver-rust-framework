use nt_string::unicode_string::NtUnicodeStr;
use wdrf_std::{NtResult, NtResultEx};
use windows_sys::Wdk::Storage::FileSystem::Minifilters::{
    FltGetFileNameInformation, FltReleaseFileNameInformation, FLTFL_FILE_NAME_PARSED_EXTENSION,
    FLTFL_FILE_NAME_PARSED_FINAL_COMPONENT, FLTFL_FILE_NAME_PARSED_PARENT_DIR,
    FLTFL_FILE_NAME_PARSED_STREAM, FLT_FILE_NAME_INFORMATION, FLT_FILE_NAME_NORMALIZED,
    FLT_FILE_NAME_OPENED,
};

use super::FltCallbackData;

#[repr(transparent)]
pub struct FileNameInformation {
    file_info: &'static mut FLT_FILE_NAME_INFORMATION,
}

unsafe impl Send for FileNameInformation {}
unsafe impl Sync for FileNameInformation {}

impl FileNameInformation {
    pub fn create(data: &FltCallbackData) -> NtResult<Self> {
        let mut file_info = core::ptr::null_mut();
        let status = unsafe {
            FltGetFileNameInformation(data.raw_struct() as _, FLT_FILE_NAME_OPENED, &mut file_info)
        };

        NtResult::from_status(status, || Self {
            file_info: unsafe { &mut *file_info },
        })
    }

    pub fn flag_final_component(&self) -> bool {
        ((self.file_info.NamesParsed as u32) & FLTFL_FILE_NAME_PARSED_FINAL_COMPONENT)
            == FLTFL_FILE_NAME_PARSED_FINAL_COMPONENT
    }

    pub fn flag_file_extension(&self) -> bool {
        ((self.file_info.NamesParsed as u32) & FLTFL_FILE_NAME_PARSED_EXTENSION)
            == FLTFL_FILE_NAME_PARSED_EXTENSION
    }

    pub fn flag_file_stream(&self) -> bool {
        ((self.file_info.NamesParsed as u32) & FLTFL_FILE_NAME_PARSED_STREAM)
            == FLTFL_FILE_NAME_PARSED_STREAM
    }

    pub fn flag_parent_dir(&self) -> bool {
        ((self.file_info.NamesParsed as u32) & FLTFL_FILE_NAME_PARSED_PARENT_DIR)
            == FLTFL_FILE_NAME_PARSED_PARENT_DIR
    }

    pub fn name(&self) -> NtUnicodeStr<'_> {
        unsafe {
            let name = self.file_info.Name;
            NtUnicodeStr::from_raw_parts(name.Buffer, name.Length, name.MaximumLength)
        }
    }

    pub fn volume(&self) -> Option<NtUnicodeStr<'_>> {
        unsafe {
            let name = self.file_info.Volume;
            if name.Length == 0 {
                None
            } else {
                Some(NtUnicodeStr::from_raw_parts(
                    name.Buffer,
                    name.Length,
                    name.MaximumLength,
                ))
            }
        }
    }

    pub fn extension(&self) -> Option<NtUnicodeStr<'_>> {
        unsafe {
            let name = self.file_info.Extension;
            if name.Length == 0 {
                None
            } else {
                Some(NtUnicodeStr::from_raw_parts(
                    name.Buffer,
                    name.Length,
                    name.MaximumLength,
                ))
            }
        }
    }

    pub fn stream(&self) -> Option<NtUnicodeStr<'_>> {
        unsafe {
            let name = self.file_info.Stream;
            if name.Length == 0 {
                None
            } else {
                Some(NtUnicodeStr::from_raw_parts(
                    name.Buffer,
                    name.Length,
                    name.MaximumLength,
                ))
            }
        }
    }

    pub fn final_component(&self) -> Option<NtUnicodeStr<'_>> {
        unsafe {
            let name = self.file_info.FinalComponent;
            if name.Length == 0 {
                None
            } else {
                Some(NtUnicodeStr::from_raw_parts(
                    name.Buffer,
                    name.Length,
                    name.MaximumLength,
                ))
            }
        }
    }

    pub fn parent_dir(&self) -> Option<NtUnicodeStr<'_>> {
        unsafe {
            let name = self.file_info.ParentDir;
            if name.Length == 0 {
                None
            } else {
                Some(NtUnicodeStr::from_raw_parts(
                    name.Buffer,
                    name.Length,
                    name.MaximumLength,
                ))
            }
        }
    }

    pub unsafe fn raw_struct(&self) -> &'static FLT_FILE_NAME_INFORMATION {
        let ptr: *const FLT_FILE_NAME_INFORMATION = self.file_info;
        &*ptr
    }
}

impl Drop for FileNameInformation {
    fn drop(&mut self) {
        unsafe {
            FltReleaseFileNameInformation(self.file_info);
        }
    }
}
