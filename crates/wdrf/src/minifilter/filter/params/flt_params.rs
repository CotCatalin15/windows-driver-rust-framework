use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_PARAMETERS;

use crate::minifilter::filter::registration::FltOperationType;

use super::{FltCreateRequest, FltQueryFileRequest, FltReadFileRequest};

pub enum FltParameters<'a> {
    Create(FltCreateRequest<'a>),
    QueryInformation(FltQueryFileRequest<'a>),
    Read(FltReadFileRequest<'a>),
}

impl<'a> FltParameters<'a> {
    pub fn new(operation_type: FltOperationType, params: &'a mut FLT_PARAMETERS) -> Self {
        unsafe {
            match operation_type {
                FltOperationType::Create => Self::Create(FltCreateRequest::new(params)),
                FltOperationType::Query => Self::QueryInformation(FltQueryFileRequest::new(params)),
                FltOperationType::Read => Self::Read(FltReadFileRequest::new(params)),
            }
        }
    }
}
