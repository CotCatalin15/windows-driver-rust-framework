use core::cell::UnsafeCell;

use wdk_sys::{
    ntddk::{KeClearEvent, KeInitializeEvent, KePulseEvent, KeSetEvent, KeWaitForSingleObject},
    IO_NO_INCREMENT, KEVENT, NTSTATUS,
    _EVENT_TYPE::{NotificationEvent, SynchronizationEvent},
    _KWAIT_REASON::Executive,
    _MODE::KernelMode,
};

use crate::{
    kmalloc::TaggedObject,
    sync::arc::{Arc, ArcExt},
    traits::DispatchSafe,
};

use super::WaitableObject;

pub struct UnsafeEvent(UnsafeCell<KEVENT>);

unsafe impl Send for UnsafeEvent {}
unsafe impl Sync for UnsafeEvent {}
unsafe impl DispatchSafe for UnsafeEvent {}

unsafe impl WaitableObject for UnsafeEvent {
    unsafe fn get_object(&self) -> *const () {
        return self.0.get().cast();
    }
}

impl TaggedObject for UnsafeEvent {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"uevt")
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    Notification,
    Synchronization,
}

impl EventType {
    fn as_wdm_value(self) -> i32 {
        match self {
            EventType::Notification => NotificationEvent,
            EventType::Synchronization => SynchronizationEvent,
        }
    }
}

impl UnsafeEvent {
    pub fn new(ev_type: EventType) -> Self {
        unsafe {
            let mut event = core::mem::zeroed::<KEVENT>();
            KeInitializeEvent(&mut event, ev_type.as_wdm_value(), false as _);

            Self(UnsafeCell::new(event))
        }
    }

    #[inline(always)]
    pub unsafe fn signal(&self) {
        unsafe {
            let _ = KeSetEvent(self.0.get(), IO_NO_INCREMENT as _, false as _);
        }
    }

    #[inline(always)]
    pub fn clear(&self) {
        unsafe {
            KeClearEvent(self.0.get());
        }
    }

    #[inline(always)]
    pub fn pulse(&self) {
        unsafe {
            KePulseEvent(self.0.get(), IO_NO_INCREMENT as _, false as _);
        }
    }
}

pub struct KEvent(UnsafeEvent);

unsafe impl Send for KEvent {}
unsafe impl Sync for KEvent {}
unsafe impl DispatchSafe for KEvent {}

impl TaggedObject for KEvent {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"evnt")
    }
}

unsafe impl WaitableObject for KEvent {
    unsafe fn get_object(&self) -> *const () {
        self.0.get_object()
    }
}

impl KEvent {
    pub fn new(ev_type: EventType) -> anyhow::Result<Arc<Self>> {
        Arc::try_create(Self(UnsafeEvent::new(ev_type)))
    }

    #[inline(always)]
    pub fn signal(&self) {
        unsafe { self.0.signal() };
    }

    #[inline(always)]
    pub fn clear(&self) {
        self.0.clear()
    }

    #[inline(always)]
    pub fn pulse(&self) {
        self.0.pulse();
    }
}
