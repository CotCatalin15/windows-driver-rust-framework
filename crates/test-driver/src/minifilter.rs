use nt_string::unicode_string::NtUnicodeStr;
use wdrf::minifilter::filter::{FileNameInformation, PreOpStatus, PreOperationVisitor};
use wdrf_std::{kmalloc::TaggedObject, nt_success};
use windows_sys::Wdk::Storage::FileSystem::{
    IoReplaceFileObjectName,
    Minifilters::{
        FltGetFileNameInformation, FltReleaseFileNameInformation, FLT_FILE_NAME_NORMALIZED,
    },
};

pub struct MinifilterPreOperation {}

impl PreOperationVisitor for MinifilterPreOperation {
    fn create<'a>(
        &self,
        data: wdrf::minifilter::filter::FltCallbackData<'a>,
        related_obj: wdrf::minifilter::filter::FltRelatedObjects<'a>,
        create: wdrf::minifilter::filter::params::FltCreateRequest<'a>,
    ) -> wdrf::minifilter::filter::PreOpStatus {
        if let Ok(info) = FileNameInformation::create(&data) {
            let name = info.name();

            maple::info!("Create file info: {name}");
        }

        IoReplaceFileObjectName(fileobject, newfilename, filenamelength);

        PreOpStatus::SuccessNoCallback
    }
}

impl TaggedObject for MinifilterPreOperation {}
