use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS;

use crate::minifilter::filter::registration::FltOperationType;

use super::{
    FltAcquireForSectionSynchronizationRequest, FltCleanupFileRequest, FltCloseFileRequest,
    FltCreateRequest, FltQueryFileInformationRequest, FltReadFileRequest,
    FltSetFileInformationRequest, FltWriteFileRequest,
};

pub enum FltParameters<'a> {
    Create(FltCreateRequest<'a>),
    Read(FltReadFileRequest<'a>),
    Write(FltWriteFileRequest<'a>),
    Cleanup(FltCleanupFileRequest),
    Close(FltCloseFileRequest),
    QueryFileInfo(FltQueryFileInformationRequest<'a>),
    SetFileInfo(FltSetFileInformationRequest<'a>),
    AcquireForSectionSync(FltAcquireForSectionSynchronizationRequest<'a>),
}

impl<'a> FltParameters<'a> {
    pub fn new(operation_type: FltOperationType, params: &'a mut FLT_PARAMETERS) -> Self {
        unsafe {
            match operation_type {
                FltOperationType::Create => Self::Create(FltCreateRequest::new(params)),
                FltOperationType::Read => Self::Read(FltReadFileRequest::new(params)),
                FltOperationType::Write => Self::Write(FltWriteFileRequest::new(params)),
                FltOperationType::Cleanup => Self::Cleanup(FltCleanupFileRequest::new()),
                FltOperationType::Close => Self::Close(FltCloseFileRequest::new()),
                FltOperationType::QueryFileInfo => {
                    Self::QueryFileInfo(FltQueryFileInformationRequest::new(params))
                }
                FltOperationType::SetFileInfo => {
                    Self::SetFileInfo(FltSetFileInformationRequest::new(params))
                }
                FltOperationType::AcquireForSectionSync => Self::AcquireForSectionSync(
                    FltAcquireForSectionSynchronizationRequest::new(params),
                ),
            }
        }
    }
}
