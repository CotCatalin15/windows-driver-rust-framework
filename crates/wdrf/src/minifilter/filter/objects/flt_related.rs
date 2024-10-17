use core::ops::Deref;

use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_RELATED_OBJECTS;

#[repr(transparent)]
pub struct FltRelatedObjects<'a>(&'a FLT_RELATED_OBJECTS);

impl<'a> FltRelatedObjects<'a> {
    pub fn new(data: *const FLT_RELATED_OBJECTS) -> Self {
        Self(unsafe { &*data })
    }
}

impl<'a> Deref for FltRelatedObjects<'a> {
    type Target = FLT_RELATED_OBJECTS;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
