use windows_sys::Wdk::System::SystemServices::{KeDelayExecutionThread, KernelMode};

use crate::time::Timeout;

pub fn delay_execution(timeout: Timeout) {
    unsafe {
        let _ = KeDelayExecutionThread(KernelMode as _, false as _, timeout.as_ptr());
    }
}
