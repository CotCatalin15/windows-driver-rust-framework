use core::any::Any;

use windows_sys::Win32::Foundation::NTSTATUS;

use crate::minifilter::filter::{
    params::{FltCreateRequest, FltQueryFileRequest, FltReadFileRequest},
    FltCallbackData, FltRelatedObjects,
};

use super::PostOpContext;

pub enum PreOpStatus {
    Complete(NTSTATUS),
    DisalowFastIO,
    Pending,
    SuccessNoCallback,
    SuccessWithCallback(Option<PostOpContext<dyn Any>>),
    Sync,
    DisallowFsFilterIo,
}

#[allow(unused_variables)]
pub trait PreOperationVisitor: 'static + Send + Sync {
    fn create<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        create: FltCreateRequest<'a>,
    ) -> PreOpStatus {
        PreOpStatus::SuccessNoCallback
    }

    fn query_file_information<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        query: FltQueryFileRequest<'a>,
    ) -> PreOpStatus {
        PreOpStatus::SuccessNoCallback
    }

    fn read<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        read: FltReadFileRequest<'a>,
    ) -> PreOpStatus {
        PreOpStatus::SuccessNoCallback
    }
}
