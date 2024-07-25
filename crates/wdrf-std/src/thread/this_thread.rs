use core::{ptr::NonNull, time::Duration};

use wdk_sys::{
    ntddk::{KeDelayExecutionThread, PsGetCurrentThreadId},
    LARGE_INTEGER, PKTHREAD,
    _MODE::KernelMode,
};

use crate::object::ArcKernelObj;

pub fn this_thread_object() -> ArcKernelObj<PKTHREAD> {
    unsafe {
        let handle: *mut PKTHREAD = PsGetCurrentThreadId() as _;

        ArcKernelObj::from_raw_object(NonNull::new(handle).unwrap(), true)
    }
}

pub fn delay_execution(duration: Duration) {
    unsafe {
        let mut timeout: LARGE_INTEGER = core::mem::zeroed();
        timeout.QuadPart = -((duration.as_nanos() / 100) as i64);

        let _ = KeDelayExecutionThread(KernelMode as _, false as _, &mut timeout);
    }
}
