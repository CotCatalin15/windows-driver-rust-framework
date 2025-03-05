use core::any::Any;

use wdrf_std::kmalloc::TaggedObject;
use windows_sys::Win32::Foundation::NTSTATUS;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::PostOpContext;

pub enum PreOpStatus<C: 'static + Send + Sync + TaggedObject> {
    Complete(NTSTATUS, usize),
    DisalowFastIO,
    Pending,
    SuccessNoCallback,
    SuccessWithCallback(Option<PostOpContext<C>>),
    Sync,
    DisallowFsFilterIo,
}

pub trait FltPreOpCallback<'a, C, PostContext>
where
    C: 'static + Sized + Sync + Send,
    PostContext: 'static + Sized + Sync + Send + TaggedObject,
{
    fn call(
        minifilter_context: &'a C,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
    ) -> PreOpStatus<PostContext>;
}
