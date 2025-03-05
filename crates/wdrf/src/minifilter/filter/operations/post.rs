use core::any::Any;

use wdrf_std::kmalloc::TaggedObject;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::{FltPreOpCallback, PostOpContext};

#[derive(Debug, Clone, Copy)]
pub enum PostOpStatus {
    FinishProcessing,
    PendProcessing,
}

pub trait FltPostOpCallback<'a>: FltPreOpCallback<'a> {
    fn call_post(
        minifilter_context: &'static Self::MinifilterContext,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
        context: Option<PostOpContext<Self::PostContext>>,
        draining: bool,
    ) -> PostOpStatus;
}
