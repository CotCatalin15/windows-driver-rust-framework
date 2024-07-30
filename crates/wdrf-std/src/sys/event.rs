use windows_sys::{
    Wdk::{
        Foundation::KEVENT,
        Storage::FileSystem::IO_NO_INCREMENT,
        System::SystemServices::{KeClearEvent, KeInitializeEvent, KePulseEvent, KeSetEvent},
    },
    Win32::System::Kernel::{NotificationEvent, SynchronizationEvent},
};

use crate::kmalloc::TaggedObject;

use super::{WaitableKernelObject, WaitableObject};

#[repr(C)]
pub struct KeEvent(KEVENT);

impl TaggedObject for KeEvent {
    fn tag() -> crate::kmalloc::MemoryTag {
        crate::kmalloc::MemoryTag::new_from_bytes(b"kmev")
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EventType {
    Notification,
    Synchronization,
}

unsafe impl Send for KeEvent {}

impl EventType {
    fn as_wdm_value(self) -> i32 {
        match self {
            EventType::Notification => NotificationEvent,
            EventType::Synchronization => SynchronizationEvent,
        }
    }
}

impl KeEvent {
    ///
    ///# Safety
    ///
    /// Moving this object will invalidate internal pointers
    /// resulting in  a BugCheck
    ///
    pub unsafe fn new() -> Self {
        unsafe { Self(core::mem::zeroed::<KEVENT>()) }
    }

    pub fn init(&self, evtype: EventType, signaled: bool) {
        unsafe {
            let event: *const KEVENT = &self.0;
            KeInitializeEvent(event as _, evtype.as_wdm_value(), signaled as _);
        }
    }

    pub fn signal(&self) {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            KeSetEvent(ptr as _, IO_NO_INCREMENT as _, false as _);
        }
    }

    pub fn signal_wait(&self) {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            KeSetEvent(ptr as _, IO_NO_INCREMENT as _, true as _);
        }
    }

    pub fn pulse(&self) {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            KePulseEvent(ptr as _, IO_NO_INCREMENT as _, false as _);
        }
    }

    pub fn pulse_wait(&self) {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            KePulseEvent(ptr as _, IO_NO_INCREMENT as _, true as _);
        }
    }

    pub fn clear(&self) {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            KeClearEvent(ptr as _);
        }
    }
}

unsafe impl WaitableObject for KeEvent {
    fn kernel_object(&self) -> &WaitableKernelObject {
        unsafe {
            let ptr: *const KEVENT = &self.0;
            &*(ptr.cast())
        }
    }
}
