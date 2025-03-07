use wdrf_std::kmalloc::TaggedObject;
use windows_sys::Win32::Foundation::NTSTATUS;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::PostOpContext;

pub enum PreOpStatus<C: 'static + Send + TaggedObject> {
    Complete(NTSTATUS, usize),
    DisalowFastIO,
    Pending,
    SuccessNoCallback,
    SuccessWithCallback(Option<PostOpContext<C>>),
    Sync,
    DisallowFsFilterIo,
}

pub trait FltPreOpCallback<'a> {
    type MinifilterContext: 'static + Sized + Sync + Send;
    type PostContext: 'static + Sized + Send + TaggedObject;

    fn call_pre(
        minifilter_context: &'a Self::MinifilterContext,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
    ) -> PreOpStatus<Self::PostContext>;
}
