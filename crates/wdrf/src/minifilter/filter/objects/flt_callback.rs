use wdrf_std::{
    object::{ArcKernelObj, NonNullKrnlResource},
    structs::{PETHREAD, PKTHREAD},
};
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FltSetCallbackDataDirty, FLTFL_CALLBACK_DATA_DIRTY, FLTFL_CALLBACK_DATA_DRAINING_IO,
        FLTFL_CALLBACK_DATA_FAST_IO_OPERATION, FLTFL_CALLBACK_DATA_FS_FILTER_OPERATION,
        FLTFL_CALLBACK_DATA_GENERATED_IO, FLTFL_CALLBACK_DATA_IRP_OPERATION,
        FLTFL_CALLBACK_DATA_POST_OPERATION, FLTFL_CALLBACK_DATA_REISSUED_IO,
        FLTFL_CALLBACK_DATA_SYSTEM_BUFFER, FLT_CALLBACK_DATA,
    },
    Win32::Foundation::NTSTATUS,
};

use super::flt_io_param::FltIoParameterBlock;

#[repr(transparent)]
pub struct FltCallbackData<'a>(&'a mut FLT_CALLBACK_DATA);

unsafe impl<'a> Send for FltCallbackData<'a> {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FilterDataOperation {
    FastIo,
    FsFilter,
    Irp,
}

impl<'a> FltCallbackData<'a> {
    pub fn new(data: *mut FLT_CALLBACK_DATA) -> Self {
        Self(unsafe { &mut *data })
    }

    pub fn set_data_dirty(&mut self) {
        unsafe {
            FltSetCallbackDataDirty(self.0);
        }
    }

    pub fn is_dirty(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_DIRTY) == FLTFL_CALLBACK_DATA_DIRTY
    }

    pub fn data_operation(&self) -> FilterDataOperation {
        FilterDataOperation::from_flags(self.0.Flags)
    }

    pub fn is_data_genrated_io(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_GENERATED_IO) == FLTFL_CALLBACK_DATA_GENERATED_IO
    }

    pub fn is_reissued(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_REISSUED_IO) == FLTFL_CALLBACK_DATA_REISSUED_IO
    }

    pub fn is_system_buffer(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_SYSTEM_BUFFER) == FLTFL_CALLBACK_DATA_SYSTEM_BUFFER
    }

    pub fn is_draining_io(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_DRAINING_IO) == FLTFL_CALLBACK_DATA_DRAINING_IO
    }

    pub fn is_post_operation(&self) -> bool {
        (self.0.Flags & FLTFL_CALLBACK_DATA_POST_OPERATION) == FLTFL_CALLBACK_DATA_POST_OPERATION
    }

    pub fn thread(&self) -> Option<ArcKernelObj<PETHREAD>> {
        let pkthread: PKTHREAD = self.0.Thread as _;
        NonNullKrnlResource::new(pkthread).map(|th| ArcKernelObj::new(th, true))
    }

    ///
    /// #Safety
    /// Let the minifilter framework set the status based on the return status
    ///
    pub unsafe fn set_status(&mut self, status: NTSTATUS, information: usize) {
        self.0.IoStatus.Anonymous.Status = status;
        self.0.IoStatus.Information = information;
    }

    pub unsafe fn raw_struct(&self) -> *mut FLT_CALLBACK_DATA {
        (self.0 as *const FLT_CALLBACK_DATA) as _
    }

    pub fn requestor_mode(&self) -> i8 {
        self.0.RequestorMode
    }

    pub fn io_params(&self) -> FltIoParameterBlock<'a> {
        FltIoParameterBlock::new(unsafe { &*self.0.Iopb })
    }
}

impl FilterDataOperation {
    pub fn from_flags(flags: u32) -> Self {
        if (flags & FLTFL_CALLBACK_DATA_FAST_IO_OPERATION) == FLTFL_CALLBACK_DATA_FAST_IO_OPERATION
        {
            Self::FastIo
        } else if (flags & FLTFL_CALLBACK_DATA_IRP_OPERATION) == FLTFL_CALLBACK_DATA_IRP_OPERATION {
            Self::Irp
        } else if (flags & FLTFL_CALLBACK_DATA_FS_FILTER_OPERATION)
            == FLTFL_CALLBACK_DATA_FS_FILTER_OPERATION
        {
            Self::FsFilter
        } else {
            panic!("Unknown file system operation flags");
        }
    }
}
