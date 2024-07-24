use wdk_sys::{
    ntddk::{KeClearEvent, KeInitializeEvent, KePulseEvent, KeSetEvent},
    IO_NO_INCREMENT, KEVENT,
    _EVENT_TYPE::{NotificationEvent, SynchronizationEvent},
};

use super::WaitableObject;

#[repr(C)]
pub struct KeEvent(KEVENT);

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
    pub fn new() -> Self {
        unsafe { Self(core::mem::zeroed::<KEVENT>()) }
    }

    pub fn init(&self, evtype: EventType) {
        unsafe {
            let event: *const KEVENT = &self.0;
            KeInitializeEvent(event as _, evtype.as_wdm_value(), false as _);
        }
    }

    pub fn init_signaled(&self, evtype: EventType) {
        unsafe {
            let event: *const KEVENT = &self.0;
            KeInitializeEvent(event as _, evtype.as_wdm_value(), true as _);
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
    unsafe fn kernel_object(&self) -> *const () {
        let ptr: *const KEVENT = &self.0;
        ptr as _
    }
}
