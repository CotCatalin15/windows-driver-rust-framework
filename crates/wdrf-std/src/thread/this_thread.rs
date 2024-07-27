use core::{ptr::NonNull, time::Duration};

use windows_sys::Wdk::System::SystemServices::{
    KeDelayExecutionThread, KernelMode, PsGetCurrentThreadId,
};

use crate::{object::ArcKernelObj, structs::PKTHREAD};

pub fn this_thread_object() -> ArcKernelObj<PKTHREAD> {
    unsafe {
        let handle: *mut PKTHREAD = PsGetCurrentThreadId() as _;

        ArcKernelObj::from_raw_object(NonNull::new(handle).unwrap(), true)
    }
}

pub fn delay_execution(duration: Duration) {
    unsafe {
        let timeout = -((duration.as_nanos() / 100) as i64);

        let _ = KeDelayExecutionThread(KernelMode as _, false as _, &timeout);
    }
}
