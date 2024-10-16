use wdrf_std::{
    object::{ArcKernelObj, NonNullKrnlResource},
    structs::PFILE_OBJECT,
};
use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_IO_PARAMETER_BLOCK;

#[repr(transparent)]
pub struct FltIoParameterBlock<'a>(&'a FLT_IO_PARAMETER_BLOCK);

impl<'a> FltIoParameterBlock<'a> {
    pub(super) fn new(data: *const FLT_IO_PARAMETER_BLOCK) -> Self {
        Self(unsafe { &*data })
    }

    pub fn irp_flags(&self) -> u32 {
        self.0.IrpFlags
    }

    pub fn operation_flags(&self) -> u8 {
        self.0.OperationFlags
    }

    pub fn file_object(&self) -> Option<ArcKernelObj<PFILE_OBJECT>> {
        NonNullKrnlResource::new(self.0.TargetFileObject).map(|obj| ArcKernelObj::new(obj, true))
    }
}
