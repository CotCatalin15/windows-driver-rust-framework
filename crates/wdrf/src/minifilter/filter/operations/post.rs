use core::any::Any;

use wdrf_std::{boxed::Box, traits::DispatchSafe};

use crate::minifilter::filter::{
    params::{FltCreateRequest, FltQueryFileRequest},
    FltCallbackData, FltRelatedObjects,
};

use super::PostOpContext;

#[derive(Debug, Clone, Copy)]
pub enum PostOpStatus {
    FinishProcessing,
    PendProcessing,
}

#[allow(unused_variables)]
pub trait PostOperationVisitor: 'static + Send + Sync + DispatchSafe {
    fn create<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        create: FltCreateRequest<'a>,
        context: Option<PostOpContext<dyn Any>>,
        draining: bool,
    ) -> PostOpStatus {
        PostOpStatus::FinishProcessing
    }

    fn query_file_information<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        create: FltQueryFileRequest<'a>,
        context: Option<PostOpContext<dyn Any>>,
        draining: bool,
    ) -> PostOpStatus {
        PostOpStatus::FinishProcessing
    }
}
