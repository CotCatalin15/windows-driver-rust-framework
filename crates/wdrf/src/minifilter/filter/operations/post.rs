use core::any::Any;

use wdrf_std::kmalloc::TaggedObject;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::PostOpContext;

#[derive(Debug, Clone, Copy)]
pub enum PostOpStatus {
    FinishProcessing,
    PendProcessing,
}

pub trait FltPostOpCallback<'a, C, PostContext>
where
    C: 'static + Sized + Sync + Send,
    PostContext: 'static + Send + Sync + TaggedObject,
{
    fn call(
        minifilter_context: &'static C,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
        context: Option<PostOpContext<PostContext>>,
        draining: bool,
    ) -> PostOpStatus;
}
