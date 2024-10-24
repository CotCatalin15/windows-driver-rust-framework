use nt_string::unicode_string::NtUnicodeStr;
use wdrf::minifilter::filter::{PreOpStatus, PreOperationVisitor};
use wdrf_std::{kmalloc::TaggedObject, nt_success};
use windows_sys::Wdk::Storage::FileSystem::Minifilters::{
    FltGetFileNameInformation, FltReleaseFileNameInformation, FLT_FILE_NAME_NORMALIZED,
};

pub struct MinifilterPreOperation {}

impl PreOperationVisitor for MinifilterPreOperation {
    fn create<'a>(
        &self,
        data: wdrf::minifilter::filter::FltCallbackData<'a>,
        related_obj: wdrf::minifilter::filter::FltRelatedObjects<'a>,
        create: wdrf::minifilter::filter::params::FltCreateRequest<'a>,
    ) -> wdrf::minifilter::filter::PreOpStatus {
        let file_object = data.io_params().file_object();

        if let Some(file_object) = file_object {
            unsafe {
                let mut ptr_file_name_info = core::ptr::null_mut();
                let status = FltGetFileNameInformation(
                    data.raw_struct(),
                    FLT_FILE_NAME_NORMALIZED,
                    &mut ptr_file_name_info,
                );
                if nt_success(status) == false {
                    return PreOpStatus::SuccessNoCallback;
                }

                let fname_info = &*ptr_file_name_info;
                let name = &fname_info.Name;
                let name = unsafe {
                    NtUnicodeStr::from_raw_parts(name.Buffer, name.Length, name.MaximumLength)
                };

                maple::info!("Create new file name: {name}");

                FltReleaseFileNameInformation(ptr_file_name_info);
            }
        }

        PreOpStatus::SuccessNoCallback
    }
}

impl TaggedObject for MinifilterPreOperation {}
