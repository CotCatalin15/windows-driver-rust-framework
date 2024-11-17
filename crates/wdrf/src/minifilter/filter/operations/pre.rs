use core::any::Any;

use windows_sys::Win32::Foundation::NTSTATUS;

use crate::minifilter::filter::{params::FltParameters, FltCallbackData, FltRelatedObjects};

use super::PostOpContext;

pub enum PreOpStatus {
    Complete(NTSTATUS, usize),
    DisalowFastIO,
    Pending,
    SuccessNoCallback,
    SuccessWithCallback(Option<PostOpContext<dyn Any>>),
    Sync,
    DisallowFsFilterIo,
}

#[allow(unused_variables)]
pub trait FltPreOpCallback: 'static + Send + Sync {
    fn callback<'a>(
        &self,
        data: FltCallbackData<'a>,
        related_obj: FltRelatedObjects<'a>,
        params: FltParameters<'a>,
    ) -> PreOpStatus;
}
