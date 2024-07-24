use core::pin::Pin;

use crate::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    sys::{
        event::{EventType, KeEvent},
        WaitableObject,
    },
    traits::DispatchSafe,
};

use super::arc::{Arc, ArcExt};

pub struct Event {
    inner: KeEvent,
}

unsafe impl DispatchSafe for Event {}
unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl TaggedObject for Event {
    fn tag() -> crate::kmalloc::MemoryTag {
        //Default kernel memory tag
        crate::kmalloc::MemoryTag::new_from_bytes(b"evnt")
    }
}

impl Event {
    pub fn try_create_arc(ev_type: EventType, signaled: bool) -> anyhow::Result<Arc<Self>> {
        let event = Arc::try_create(Self {
            inner: unsafe { KeEvent::new() },
        })?;
        event.inner.init(ev_type, signaled);

        Ok(event)
    }

    pub fn try_create_box(ev_type: EventType, signaled: bool) -> anyhow::Result<Pin<Box<Self>>> {
        let pb = Box::try_pin(Self {
            inner: unsafe { KeEvent::new() },
        })?;
        pb.inner.init(ev_type, signaled);

        Ok(pb)
    }

    #[inline(always)]
    pub fn signal(&self) {
        self.inner.signal();
    }

    #[inline(always)]
    pub fn signal_wait(&self) {
        self.inner.signal_wait();
    }

    #[inline(always)]
    pub fn pulse(&self) {
        self.inner.pulse();
    }

    #[inline(always)]
    pub fn pulse_wait(&self) {
        self.inner.pulse_wait();
    }

    #[inline(always)]
    pub fn clear(&self) {
        self.inner.clear();
    }
}

unsafe impl WaitableObject for Event {
    #[inline]
    unsafe fn kernel_object(&self) -> *const () {
        self.inner.kernel_object()
    }

    #[inline]
    fn wait(&self) -> crate::sys::WaitResponse {
        self.inner.wait()
    }

    #[inline]
    fn wait_for(&self, duration: core::time::Duration) -> crate::sys::WaitResponse {
        self.inner.wait_for(duration)
    }

    #[inline]
    fn wait_status(&self) -> crate::sys::WaitResponse {
        self.inner.wait_status()
    }
}
