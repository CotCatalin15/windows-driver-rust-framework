use windows_sys::Wdk::{
    Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION,
    System::SystemServices::{IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_READ, IRP_MJ_WRITE},
};

use crate::minifilter::structs::IRP_MJ_OPERATION_END;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FltOperationType {
    Create = IRP_MJ_CREATE as _,
    Read = IRP_MJ_READ as _,
    Write = IRP_MJ_WRITE as _,
    Close = IRP_MJ_CLOSE as _,
}

pub struct FltOperationEntry {
    op: FltOperationType,
    flags: u32,
}

impl FltOperationEntry {
    pub fn new(op: FltOperationType, flags: u32) -> Self {
        Self { op, flags }
    }

    pub(crate) fn convert_to_registry(&self) -> FLT_OPERATION_REGISTRATION {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.op as u8,
            Flags: self.flags,
            PreOperation: None,
            PostOperation: None,
            Reserved1: core::ptr::null_mut(),
        }
    }

    pub(crate) unsafe fn create_end_entry() -> FLT_OPERATION_REGISTRATION {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: IRP_MJ_OPERATION_END as _,
            ..core::mem::zeroed()
        }
    }
}
