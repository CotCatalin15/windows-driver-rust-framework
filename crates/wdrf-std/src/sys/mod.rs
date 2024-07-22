use core::time::Duration;

use wdk_sys::{
    ntddk::KeWaitForSingleObject, LARGE_INTEGER, NTSTATUS, STATUS_ALERTED, STATUS_SUCCESS,
    STATUS_TIMEOUT, STATUS_USER_APC, _KWAIT_REASON::Executive, _MODE::KernelMode,
};

pub mod event;
pub mod mutex;
pub mod semaphore;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WaitResponse {
    Success,
    Alerted,
    UserAPC,
    Timeout,
    Other(NTSTATUS),
}

impl WaitResponse {
    pub fn from_ntstatus(status: NTSTATUS) -> Self {
        match status {
            STATUS_SUCCESS => Self::Success,
            STATUS_ALERTED => Self::Alerted,
            STATUS_USER_APC => Self::UserAPC,
            STATUS_TIMEOUT => Self::Timeout,
            _ => Self::Other(status),
        }
    }
}

pub unsafe trait WaitableObject {
    unsafe fn get_object(&self) -> *const ();

    fn wait(&self) -> WaitResponse {
        unsafe {
            let status = KeWaitForSingleObject(
                self.get_object() as _,
                Executive,
                KernelMode as _,
                false as _,
                core::ptr::null_mut(),
            );

            WaitResponse::from_ntstatus(status)
        }
    }

    fn wait_for(&self, duration: Duration) -> WaitResponse {
        unsafe {
            let mut timeout: LARGE_INTEGER = core::mem::zeroed();
            timeout.QuadPart = core::mem::transmute((duration.as_nanos() / 100) as u64);

            let status = KeWaitForSingleObject(
                self.get_object() as _,
                Executive,
                KernelMode as _,
                false as _,
                &mut timeout,
            );

            WaitResponse::from_ntstatus(status)
        }
    }
}
