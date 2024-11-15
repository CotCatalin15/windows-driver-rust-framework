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

impl PreOperationVisitor for EmptyFltOperationsVisitor {}
impl PostOperationVisitor for EmptyFltOperationsVisitor {}
impl FilterOperationVisitor for EmptyFltOperationsVisitor {}

impl TaggedObject for EmptyFltOperationsVisitor {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        MemoryTag::new_from_bytes(b"flte")
    }
}
