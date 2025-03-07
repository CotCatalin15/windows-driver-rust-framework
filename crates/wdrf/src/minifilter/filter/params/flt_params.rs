use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS;

use crate::minifilter::filter::registration::FltOperationType;

use super::{
    FltCloseFileRequest, FltCreateRequest, FltQueryFileRequest, FltReadFileRequest,
    FltWriteFileRequest,
};

pub enum FltParameters<'a> {
    Create(FltCreateRequest<'a>),
    Read(FltReadFileRequest<'a>),
    Write(FltWriteFileRequest<'a>),
    Close(FltCloseFileRequest),
}

impl<'a> FltParameters<'a> {
    pub fn new(operation_type: FltOperationType, params: &'a mut FLT_PARAMETERS) -> Self {
        unsafe {
            match operation_type {
                FltOperationType::Create => Self::Create(FltCreateRequest::new(params)),
                FltOperationType::Read => Self::Read(FltReadFileRequest::new(params)),
                FltOperationType::Write => Self::Write(FltWriteFileRequest::new(params)),
                FltOperationType::Close => Self::Close(FltCloseFileRequest),
                _ => todo!(),
            }
        }
    }
}
