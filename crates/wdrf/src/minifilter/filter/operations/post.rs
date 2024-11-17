use core::any::Any;

use wdrf_std::traits::DispatchSafe;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::PostOpContext;

#[derive(Debug, Clone, Copy)]
pub enum PostOpStatus {
    FinishProcessing,
    PendProcessing,
}

pub trait FltPostOpCallback: 'static + Send + Sync + DispatchSafe {
    fn callback<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
        context: Option<PostOpContext<dyn Any>>,
        draining: bool,
    ) -> PostOpStatus;
}
