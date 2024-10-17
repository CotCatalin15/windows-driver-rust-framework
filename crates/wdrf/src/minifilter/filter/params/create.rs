use windows_sys::Wdk::{
    Foundation::IO_SECURITY_CONTEXT,
    Storage::FileSystem::Minifilters::{FLT_PARAMETERS, FLT_PARAMETERS_4},
};

// TODO: In a galaxy far, far away... bitflags! {} was used, and no one returned raw u16/u32 flags again.
pub struct FltCreateRequest<'a> {
    create_params: &'a mut FLT_PARAMETERS_4,
}

impl<'a> FltCreateRequest<'a> {
    pub unsafe fn new(params: &'a mut FLT_PARAMETERS) -> Self {
        Self {
            create_params: &mut params.Create,
        }
    }

    pub unsafe fn raw_create(&'a mut self) -> &'a mut FLT_PARAMETERS_4 {
        self.create_params
    }

    pub unsafe fn security_context(&'a mut self) -> &'a mut IO_SECURITY_CONTEXT {
        unsafe { &mut *self.create_params.SecurityContext }
    }

    pub fn desired_access(&self) -> u32 {
        unsafe { *self.create_params.SecurityContext }.DesiredAccess
    }

    pub fn options(&self) -> u32 {
        self.create_params.Options
    }

    pub fn attributes(&self) -> u16 {
        self.create_params.FileAttributes
    }

    pub fn share_access(&self) -> u16 {
        self.create_params.ShareAccess
    }

    pub fn ea_buffer(&self) -> Option<&'a [u8]> {
        if self.create_params.EaLength == 0 {
            None
        } else {
            unsafe {
                Some(core::slice::from_raw_parts(
                    self.create_params.EaBuffer as _,
                    self.create_params.EaLength as _,
                ))
            }
        }
    }
}
