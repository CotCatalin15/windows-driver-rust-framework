use windows_sys::Wdk::Storage::FileSystem::{
    Minifilters::FLT_PARAMETERS, FS_FILTER_SECTION_SYNC_OUTPUT, FS_FILTER_SECTION_SYNC_TYPE,
};

#[repr(C)]
#[allow(non_snake_case)]
pub struct FltAcquireForSectionSynchronizationParameter {
    pub SyncType: FS_FILTER_SECTION_SYNC_TYPE,
    pub PageProtection: u32,
    pub OutputInformation: *mut FS_FILTER_SECTION_SYNC_OUTPUT,
    pub Flags: u32,
    pub AllocationAttributes: u32,
}

pub struct FltAcquireForSectionSynchronizationRequest<'a> {
    param: &'a FltAcquireForSectionSynchronizationParameter,
}

impl<'a> FltAcquireForSectionSynchronizationRequest<'a> {
    pub fn new(params: &'a FLT_PARAMETERS) -> Self {
        unsafe {
            let query_ptr = &params.AcquireForSectionSynchronization as *const _
                as *const FltAcquireForSectionSynchronizationParameter;
            Self { param: &*query_ptr }
        }
    }

    pub fn sync_type(&self) -> FS_FILTER_SECTION_SYNC_TYPE {
        self.param.SyncType
    }

    pub fn page_protection(&self) -> u32 {
        self.param.PageProtection
    }

    pub fn output_information(&self) -> Option<&'a FS_FILTER_SECTION_SYNC_OUTPUT> {
        unsafe { self.param.OutputInformation.as_ref() }
    }

    pub fn flags(&self) -> u32 {
        self.param.Flags
    }

    pub fn allocation_attributes(&self) -> u32 {
        self.param.AllocationAttributes
    }
}
