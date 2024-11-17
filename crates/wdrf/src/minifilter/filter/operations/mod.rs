mod context;
mod filter;
mod post;
mod pre;

pub use context::*;
pub use filter::*;
pub use post::*;
pub use pre::*;

use wdrf_std::{
    kmalloc::{MemoryTag, TaggedObject},
    traits::DispatchSafe,
};

pub struct EmptyFltOperationsVisitor {}
unsafe impl DispatchSafe for EmptyFltOperationsVisitor {}

impl FltPreOpCallback for EmptyFltOperationsVisitor {
    fn callback<'a>(
        &self,
        _data: super::FltCallbackData<'a>,
        _related_obj: super::FltRelatedObjects<'a>,
        _params: super::params::FltParameters<'a>,
    ) -> PreOpStatus {
        PreOpStatus::SuccessNoCallback
    }
}

impl FltPostOpCallback for EmptyFltOperationsVisitor {
    fn callback<'a>(
        &self,
        _data: super::FltCallbackData<'a>,
        _related_obj: super::FltRelatedObjects<'a>,
        _params: super::params::FltParameters<'a>,
        _context: Option<PostOpContext<dyn core::any::Any>>,
        _draining: bool,
    ) -> PostOpStatus {
        PostOpStatus::FinishProcessing
    }
}

impl FilterOperationVisitor for EmptyFltOperationsVisitor {
    fn unload(&self, _mandatory: bool) -> UnloadStatus {
        UnloadStatus::Unload
    }
}

impl TaggedObject for EmptyFltOperationsVisitor {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        MemoryTag::new_from_bytes(b"flte")
    }
}
