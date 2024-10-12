#![allow(dead_code)]

pub mod event;
//pub(crate) mod mutex;
pub mod semaphore;

use core::time::Duration;

use windows_sys::{
    Wdk::System::SystemServices::{
        Executive, KeWaitForMultipleObjects, KeWaitForSingleObject, KernelMode,
    },
    Win32::{
        Foundation::{
            NTSTATUS, STATUS_ABANDONED_WAIT_0, STATUS_ABANDONED_WAIT_63,
            STATUS_MUTANT_LIMIT_EXCEEDED, STATUS_SUCCESS, STATUS_TIMEOUT, STATUS_WAIT_0,
            STATUS_WAIT_63,
        },
        System::Kernel::{WaitAll, WaitAny},
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WaitResponse {
    Success,
    Timeout,
    MutantLimitExceeded,
    Object(u32),
    Abandoned(u32),
}

impl WaitResponse {
    pub fn from_ntstatus(status: NTSTATUS) -> Self {
        match status {
            STATUS_SUCCESS => Self::Success,
            STATUS_TIMEOUT => Self::Timeout,
            STATUS_WAIT_0..=STATUS_WAIT_63 => Self::Object((status - STATUS_WAIT_0) as _),
            STATUS_ABANDONED_WAIT_0..=STATUS_ABANDONED_WAIT_63 => {
                Self::Abandoned((status - STATUS_ABANDONED_WAIT_0) as _)
            }
            STATUS_MUTANT_LIMIT_EXCEEDED => Self::MutantLimitExceeded,
            _ => panic!("Unknown KeWaitForSingleObject status: {}", status),
        }
    }
}

#[repr(C)]
pub struct WaitableKernelObject;

///
/// # Safety
///
/// Its safe to implement as long as kernel_object returns a valid
/// object that can be used with KeWait
///
pub unsafe trait WaitableObject {
    fn kernel_object(&self) -> &WaitableKernelObject;

    //#[cfg_attr(feature = "irql-check", irql_check(irql = APC_LELVEL))]
    fn wait(&self) -> WaitResponse {
        unsafe {
            let ptr: *const WaitableKernelObject = self.kernel_object();

            let status = KeWaitForSingleObject(
                ptr as _,
                Executive,
                KernelMode as _,
                false as _,
                core::ptr::null_mut(),
            );

            WaitResponse::from_ntstatus(status)
        }
    }

    //#[cfg_attr(feature = "irql-check", irql_check(irql = APC_LELVEL))]
    fn wait_for(&self, duration: Duration) -> WaitResponse {
        unsafe {
            let timeout: i64 = -((duration.as_nanos() / 100) as i64);

            let ptr: *const WaitableKernelObject = self.kernel_object();
            let status =
                KeWaitForSingleObject(ptr.cast(), Executive, KernelMode as _, false as _, &timeout);

            WaitResponse::from_ntstatus(status)
        }
    }

    //#[cfg_attr(feature = "irql-check", irql_check(irql = DISPATCH_LEVEL))]
    fn wait_status(&self) -> WaitResponse {
        unsafe {
            let timeout: i64 = 0;

            let ptr: *const WaitableKernelObject = self.kernel_object();
            let status =
                KeWaitForSingleObject(ptr as _, Executive, KernelMode as _, false as _, &timeout);

            WaitResponse::from_ntstatus(status)
        }
    }
}

pub struct DpcWaitError;

pub struct MultiWaitArray<'a> {
    wait_array: &'a [&'a WaitableKernelObject],
    wait_all: i32,
}

impl<'a> MultiWaitArray<'a> {
    #[inline(always)]
    pub fn new(objects: &'a [&'a WaitableKernelObject]) -> Self {
        Self {
            wait_array: objects,
            wait_all: WaitAny,
        }
    }

    #[inline(always)]
    pub fn new_wait_all(objects: &'a [&'a WaitableKernelObject]) -> Self {
        Self {
            wait_array: objects,
            wait_all: WaitAll,
        }
    }
}

unsafe impl<'a> WaitableObject for MultiWaitArray<'a> {
    fn kernel_object(&self) -> &WaitableKernelObject {
        panic!("Cannot get kernel_object from a waitable array")
    }

    fn wait(&self) -> WaitResponse {
        unsafe {
            let array = self.wait_array.as_ptr();

            let status = KeWaitForMultipleObjects(
                self.wait_array.len() as _,
                array as _,
                self.wait_all,
                Executive,
                KernelMode as _,
                false as _,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            WaitResponse::from_ntstatus(status)
        }
    }

    fn wait_for(&self, duration: Duration) -> WaitResponse {
        unsafe {
            let timeout = -((duration.as_nanos() / 100) as i64);

            let array = self.wait_array.as_ptr();

            let status = KeWaitForMultipleObjects(
                self.wait_array.len() as _,
                array as _,
                self.wait_all,
                Executive,
                KernelMode as _,
                false as _,
                &timeout,
                core::ptr::null_mut(),
            );

            WaitResponse::from_ntstatus(status)
        }
    }

    fn wait_status(&self) -> WaitResponse {
        unsafe {
            let timeout = 0;

            let array = self.wait_array.as_ptr();

            let status = KeWaitForMultipleObjects(
                self.wait_array.len() as _,
                array as _,
                self.wait_all,
                Executive,
                KernelMode as _,
                false as _,
                &timeout,
                core::ptr::null_mut(),
            );

            WaitResponse::from_ntstatus(status)
        }
    }
}
