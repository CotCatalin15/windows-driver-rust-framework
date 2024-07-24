use core::ptr::NonNull;

use wdk_sys::ntddk::{ObfDereferenceObject, ObfReferenceObject};
use wdk_sys::{ntddk::ObReferenceObjectByHandle, _MODE::KernelMode};

use wdk_sys::{
    HANDLE, PACCESS_TOKEN, PFILE_OBJECT, PKENLISTMENT, PKEVENT, PKPROCESS, PKRESOURCEMANAGER,
    PKSEMAPHORE, PKTHREAD, PKTM, PKTRANSACTION, STATUS_OBJECT_TYPE_MISMATCH,
};

use crate::sys::WaitableObject;
use crate::{NtResult, NtResultEx, NtStatusError};

use super::handle::Handle;
use super::KernelObjectType;

pub trait KernelResource {
    fn object_type() -> KernelObjectType;
}

pub struct ArcKernelObj<T: KernelResource> {
    obj: NonNull<T>,
}

impl<T> ArcKernelObj<T>
where
    T: KernelResource,
{
    pub fn from_handle(handle: &Handle, access: u32) -> NtResult<Self> {
        if !handle
            .object_type()
            .is_some_and(|val| val == T::object_type())
        {
            Err(NtStatusError::Status(STATUS_OBJECT_TYPE_MISMATCH))
        } else {
            unsafe { Self::from_raw_handle(handle.raw_handle(), access) }
        }
    }

    pub unsafe fn from_raw_handle(raw_handle: HANDLE, access: u32) -> NtResult<Self> {
        unsafe {
            let mut obj_ptr = core::ptr::null_mut();

            let status = ObReferenceObjectByHandle(
                raw_handle,
                access,
                T::object_type().into_kernel_object_type(),
                KernelMode as _,
                &mut obj_ptr,
                core::ptr::null_mut(),
            );

            let non_null = NonNull::new(obj_ptr);

            if let None = non_null {
                Err(NtStatusError::Status(STATUS_OBJECT_TYPE_MISMATCH))
            } else {
                NtResult::from_status(status, || Self {
                    obj: non_null.unwrap().cast(),
                })
            }
        }
    }

    pub unsafe fn raw_obj(&self) -> *mut T {
        self.obj.as_ptr()
    }
}

impl<T> Drop for ArcKernelObj<T>
where
    T: KernelResource,
{
    fn drop(&mut self) {
        unsafe {
            let _ = ObfDereferenceObject(self.obj.as_ptr().cast());
        }
    }
}

impl<T> Clone for ArcKernelObj<T>
where
    T: KernelResource,
{
    fn clone(&self) -> Self {
        unsafe {
            let _ = ObfReferenceObject(self.obj.as_ptr().cast());
        }
        Self {
            obj: self.obj.clone(),
        }
    }
}

macro_rules! impl_kernel_resource {
    ($o:ident, $t:expr) => {
        impl KernelResource for $o {
            fn object_type() -> KernelObjectType {
                $t
            }
        }
    };
}

impl_kernel_resource!(PKEVENT, KernelObjectType::Event);
impl_kernel_resource!(PKSEMAPHORE, KernelObjectType::Semaphore);
impl_kernel_resource!(PFILE_OBJECT, KernelObjectType::File);

impl_kernel_resource!(PKPROCESS, KernelObjectType::Process);
impl_kernel_resource!(PKTHREAD, KernelObjectType::Thread);

impl_kernel_resource!(PACCESS_TOKEN, KernelObjectType::Token);

impl_kernel_resource!(PKENLISTMENT, KernelObjectType::Enlistment);

impl_kernel_resource!(PKRESOURCEMANAGER, KernelObjectType::ResourceManager);
impl_kernel_resource!(PKTM, KernelObjectType::TranscationManager);
impl_kernel_resource!(PKTRANSACTION, KernelObjectType::Transcation);

unsafe impl WaitableObject for ArcKernelObj<PKTHREAD> {
    fn kernel_object(&self) -> &crate::sys::WaitableKernelObject {
        unsafe { self.obj.cast().as_ref() }
    }
}

unsafe impl WaitableObject for ArcKernelObj<PKPROCESS> {
    fn kernel_object(&self) -> &crate::sys::WaitableKernelObject {
        unsafe { self.obj.cast().as_ref() }
    }
}
