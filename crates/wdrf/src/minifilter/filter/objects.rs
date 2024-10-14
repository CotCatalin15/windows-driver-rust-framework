use windows_sys::Wdk::Storage::FileSystem::Minifilters::{
    FLTFL_CALLBACK_DATA_DIRTY, FLTFL_CALLBACK_DATA_FAST_IO_OPERATION,
    FLTFL_CALLBACK_DATA_FS_FILTER_OPERATION, FLTFL_CALLBACK_DATA_GENERATED_IO,
    FLTFL_CALLBACK_DATA_IRP_OPERATION, FLTFL_CALLBACK_DATA_REISSUED_IO,
    FLTFL_CALLBACK_DATA_SYSTEM_BUFFER, FLT_CALLBACK_DATA,
};

#[repr(transparent)]
pub struct FltCallbackData<'a>(&'a mut FLT_CALLBACK_DATA);

pub enum FilterDataOperation {
    FastIo,
    FsFilter,
    Irp,
}

impl<'a> FltCallbackData<'a> {
    pub(super) fn new(data: *mut FLT_CALLBACK_DATA) -> Self {
        Self(unsafe { &mut *data })
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
