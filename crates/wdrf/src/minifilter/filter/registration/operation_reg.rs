use windows_sys::Wdk::{
    Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION,
    System::SystemServices::{
        IRP_MJ_CLEANUP, IRP_MJ_CLOSE, IRP_MJ_CREATE, IRP_MJ_QUERY_INFORMATION, IRP_MJ_READ,
        IRP_MJ_SET_INFORMATION, IRP_MJ_WRITE,
    },
};

use crate::minifilter::structs::IRP_MJ_OPERATION_END;

pub const IRP_MJ_ACQUIRE_FOR_SECTION_SYNCHRONIZATION: u32 =
    unsafe { core::mem::transmute::<i8, u8>(-1i8) as u32 };

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FltOperationType {
    Create = IRP_MJ_CREATE as u8,
    Read = IRP_MJ_READ as u8,
    Write = IRP_MJ_WRITE as u8,
    Cleanup = IRP_MJ_CLEANUP as u8,
    Close = IRP_MJ_CLOSE as u8,
    QueryFileInfo = IRP_MJ_QUERY_INFORMATION as u8,
    SetFileInfo = IRP_MJ_SET_INFORMATION as u8,
    AcquireForSectionSync = IRP_MJ_ACQUIRE_FOR_SECTION_SYNCHRONIZATION as u8,
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
