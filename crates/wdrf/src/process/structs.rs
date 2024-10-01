use nt_string::unicode_string::NtUnicodeStr;
use wdrf_std::{object::NonNullKrnlResource, structs::PFILE_OBJECT};
use windows_sys::{
    Wdk::System::SystemServices::PS_CREATE_NOTIFY_INFO,
    Win32::{Foundation::HANDLE, System::WindowsProgramming::CLIENT_ID},
};

pub struct PsCreateNotifyInfo<'a> {
    pub flags: u32,
    pub parent_pid: HANDLE,
    pub client_id: CLIENT_ID,
    pub file_object: Option<NonNullKrnlResource<PFILE_OBJECT>>,
    pub image_file_name: Option<NtUnicodeStr<'a>>,
    pub command_line: Option<NtUnicodeStr<'a>>,
}

impl<'a> PsCreateNotifyInfo<'a> {
    pub(super) fn new(raw_info: &'a mut PS_CREATE_NOTIFY_INFO) -> Self {
        Self {
            flags: unsafe { raw_info.Anonymous.Flags },
            parent_pid: raw_info.ParentProcessId,
            client_id: raw_info.CreatingThreadId,
            file_object: NonNullKrnlResource::new(raw_info.FileObject),
            image_file_name: if raw_info.ImageFileName.is_null() {
                None
            } else {
                let image = raw_info.ImageFileName;
                unsafe {
                    Some(NtUnicodeStr::from_raw_parts(
                        (*image).Buffer,
                        (*image).Length,
                        (*image).MaximumLength,
                    ))
                }
            },
            command_line: if raw_info.CommandLine.is_null() {
                None
            } else {
                let cmd = raw_info.CommandLine;
                unsafe {
                    Some(NtUnicodeStr::from_raw_parts(
                        (*cmd).Buffer,
                        (*cmd).Length,
                        (*cmd).MaximumLength,
                    ))
                }
            },
        }
    }

    pub fn file_open_name_available(&self) -> bool {
        (self.flags & (1 << 0)) != 0
    }

    pub fn is_subsystem_process(&self) -> bool {
        (self.flags & (1 << 1)) != 0
    }

    pub fn reserved(&self) -> u32 {
        (self.flags >> 2) & 0x3FFFFFFF
    }
}
