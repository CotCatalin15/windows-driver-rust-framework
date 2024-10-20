mod filter;
mod post;
mod pre;

pub use filter::*;
pub use post::*;
pub use pre::*;
use wdrf_std::kmalloc::{MemoryTag, TaggedObject};

pub struct EmptyFltOperationsVisitor {}

impl PreOperationVisitor for EmptyFltOperationsVisitor {}
impl PostOperationVisitor for EmptyFltOperationsVisitor {}
impl FilterOperationVisitor for EmptyFltOperationsVisitor {}

impl TaggedObject for EmptyFltOperationsVisitor {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        MemoryTag::new_from_bytes(b"flte")
    }
}
