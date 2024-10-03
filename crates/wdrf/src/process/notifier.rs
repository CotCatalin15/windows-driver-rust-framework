use core::ops::{Deref, DerefMut};

use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
};

use super::{
    process_create_notifier::{start_collector, stop_collector, PsCreateNotifyCallback},
    ProcessCollectorError,
};

pub struct PsNotifierRegistration<CB: PsCreateNotifyCallback> {
    callback: Box<CB>,
}

impl<CB: PsCreateNotifyCallback + TaggedObject> PsNotifierRegistration<CB> {
    pub fn try_create(callback: CB) -> anyhow::Result<Self, ProcessCollectorError> {
        let callback = Box::try_create(callback).map_err(|_| ProcessCollectorError::NoMemory)?;

        Ok(Self { callback })
    }
}

impl<CB: PsCreateNotifyCallback> PsNotifierRegistration<CB> {
    pub fn try_start(&self) -> anyhow::Result<(), ProcessCollectorError> {
        unsafe {
            let ptr: *const CB = self.callback.as_ref();
            let callback: &'static CB = &*ptr;
            start_collector(callback).map_err(|e| ProcessCollectorError::NtStatus(e))
        }
    }

    pub fn try_stop(&self) -> anyhow::Result<(), ProcessCollectorError> {
        unsafe { stop_collector().map_err(|e| ProcessCollectorError::NtStatus(e)) }
    }
}

impl<CB: PsCreateNotifyCallback> Deref for PsNotifierRegistration<CB> {
    type Target = CB;
    fn deref(&self) -> &Self::Target {
        &self.callback
    }
}

impl<CB: PsCreateNotifyCallback> DerefMut for PsNotifierRegistration<CB> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.callback
    }
}

impl<CB: PsCreateNotifyCallback> Drop for PsNotifierRegistration<CB> {
    fn drop(&mut self) {
        let _ = self.try_stop();
    }
}
