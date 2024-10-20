use windows_sys::Wdk::{
    Storage::FileSystem::Minifilters::{
        FLT_OPERATION_REGISTRATION, PFLT_POST_OPERATION_CALLBACK, PFLT_PRE_OPERATION_CALLBACK,
    },
    System::SystemServices::{IRP_MJ_CREATE, IRP_MJ_QUERY_INFORMATION},
};

use crate::minifilter::{
    filter::{
        flt_op_callbacks::{
            flt_create_pre_op_implementation, flt_query_information_pre_op_implementation,
        },
        PostOperationVisitor, PreOperationVisitor,
    },
    structs::IRP_MJ_OPERATION_END,
};

#[derive(Debug, Clone, Copy)]
pub enum FltOperationType {
    Create,
    Query,
}

pub struct FltOperationEntry {
    op: FltOperationType,
    post_op: bool,
    flags: u32,
}

impl FltOperationEntry {
    pub fn new(op: FltOperationType, flags: u32, has_post: bool) -> Self {
        Self {
            op,
            post_op: has_post,
            flags,
        }
    }

    pub unsafe fn convert_to_registry<Pre: PreOperationVisitor, Post: PostOperationVisitor>(
        &self,
    ) -> FLT_OPERATION_REGISTRATION {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.op.as_irp_mj(),
            Flags: self.flags,
            PreOperation: self.op.as_pre_op_callback::<Pre>(),
            PostOperation: if self.post_op {
                self.op.as_post_op_callback::<Post>()
            } else {
                None
            },
            Reserved1: core::ptr::null_mut(),
        }
    }

    pub unsafe fn create_end_entry() -> FLT_OPERATION_REGISTRATION {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: IRP_MJ_OPERATION_END as _,
            ..core::mem::zeroed()
        }
    }
}

impl FltOperationType {
    pub fn as_irp_mj(self) -> u8 {
        match self {
            FltOperationType::Create => IRP_MJ_CREATE as _,
            FltOperationType::Query => IRP_MJ_QUERY_INFORMATION as _,
        }
    }

    pub unsafe fn as_pre_op_callback<V: PreOperationVisitor>(self) -> PFLT_PRE_OPERATION_CALLBACK {
        match self {
            FltOperationType::Create => Some(flt_create_pre_op_implementation::<V>),
            FltOperationType::Query => Some(flt_query_information_pre_op_implementation::<V>),
        }
    }

    pub unsafe fn as_post_op_callback<V: PostOperationVisitor>(
        self,
    ) -> PFLT_POST_OPERATION_CALLBACK {
        match self {
            FltOperationType::Create => todo!(),
            FltOperationType::Query => todo!(),
        }
    }
}
