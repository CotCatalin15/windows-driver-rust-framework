use nt_string::unicode_string::NtUnicodeStr;
use wdrf::minifilter::filter::{FileNameInformation, PreOpStatus, PreOperationVisitor};
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
        PreOpStatus::SuccessNoCallback
    }

    fn read<'a>(
        &self,
        data: wdrf::minifilter::filter::FltCallbackData<'a>,
        related_obj: wdrf::minifilter::filter::FltRelatedObjects<'a>,
        read: wdrf::minifilter::filter::params::FltReadFileRequest<'a>,
    ) -> PreOpStatus {
        if let Ok(info) = FileNameInformation::create(&data) {
            let name = info.name();
            maple::info!(
                "Read file info: {name}, buffer: {:?}",
                read.read_buffer().as_ptr()
            );
        }

        PreOpStatus::SuccessNoCallback
    }
}

impl TaggedObject for MinifilterPreOperation {}
