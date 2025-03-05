use windows_sys::Wdk::{
    Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION,
    System::SystemServices::{IRP_MJ_CREATE, IRP_MJ_QUERY_INFORMATION, IRP_MJ_READ},
};

use crate::minifilter::structs::IRP_MJ_OPERATION_END;

#[derive(Debug, Clone, Copy)]
pub enum FltOperationType {
    Create,
    Query,
    Read,
}

pub struct FltOperationEntry {
    op: FltOperationType,
    flags: u32,
}

impl FltOperationEntry {
    pub fn new(op: FltOperationType, flags: u32) -> Self {
        Self { op, flags }
    }

    pub fn convert_to_registry(&self) -> FLT_OPERATION_REGISTRATION {
        FLT_OPERATION_REGISTRATION {
            MajorFunction: self.op.as_irp_mj(),
            Flags: self.flags,
            PreOperation: None,
            PostOperation: None,
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
            FltOperationType::Read => IRP_MJ_READ as _,
        }
    }

    pub fn from_irp_mj(irp_mj: u8) -> Self {
        match irp_mj as u32 {
            IRP_MJ_CREATE => FltOperationType::Create,
            IRP_MJ_QUERY_INFORMATION => FltOperationType::Query,
            IRP_MJ_READ => FltOperationType::Read,
            _ => panic!("Unknown irp mj"),
        }
    }
}
