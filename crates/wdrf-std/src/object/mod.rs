pub mod attribute;
pub mod handle;

use handle::Handle;
use windows_sys::Wdk::Foundation::POBJECT_TYPE;
use windows_sys::Wdk::System::SystemServices::{
    KernelMode, ObReferenceObjectByHandle, ObfDereferenceObject, ObfReferenceObject, UserMode,
};
use windows_sys::Win32::Foundation::{HANDLE, STATUS_OBJECT_TYPE_MISMATCH};

use core::ffi::c_void;

use crate::constants::{
    ExEventObjectType, ExSemaphoreObjectType, IoFileObjectType, PsProcessType, PsThreadType,
    SeTokenObjectType, TmEnlistmentObjectType, TmResourceManagerObjectType,
    TmTransactionManagerObjectType, TmTransactionObjectType,
};
use crate::structs::PKENLISTMENT;
use crate::structs::PKEVENT;
use crate::structs::PKPROCESS;
use crate::structs::PKSEMAPHORE;
use crate::structs::PKTM;
use crate::structs::PKTRANSACTION;
use crate::structs::PPROCESS_ACCESS_TOKEN;
use crate::structs::PRKRESOURCEMANAGER;
use crate::structs::{PFILE_OBJECT, PKTHREAD};
use crate::sys::WaitableObject;
use crate::traits::DispatchSafe;
use crate::{NtResult, NtResultEx, NtStatusError};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KernelObjectType {
    Event,
    Semaphore,
    File,
    Process,
    Thread,
    Token,
    Enlistment,
    ResourceManager,
    TranscationManager,
    Transcation,
}

impl KernelObjectType {
    pub fn into_kernel_object_type(self) -> POBJECT_TYPE {
        unsafe {
            match self {
                KernelObjectType::Event => *ExEventObjectType,
                KernelObjectType::Semaphore => *ExSemaphoreObjectType,
                KernelObjectType::File => *IoFileObjectType,
                KernelObjectType::Process => *PsProcessType,
                KernelObjectType::Thread => *PsThreadType,
                KernelObjectType::Token => *SeTokenObjectType,
                KernelObjectType::Enlistment => *TmEnlistmentObjectType,
                KernelObjectType::ResourceManager => *TmResourceManagerObjectType,
                KernelObjectType::TranscationManager => *TmTransactionManagerObjectType,
                KernelObjectType::Transcation => *TmTransactionObjectType,
            }
        }
    }
}

pub trait KernelResource: Clone {
    fn object_type() -> KernelObjectType;

    fn as_mut_ptr(&self) -> *mut c_void;
    unsafe fn from_raw_ptr(ptr: *mut c_void) -> Self;
}

#[derive(Clone, Copy)]
pub struct NonNullKrnlResource<T: KernelResource> {
    resource: T,
}

impl<T> NonNullKrnlResource<T>
where
    T: KernelResource,
{
    pub fn new(resource: T) -> Option<Self> {
        if resource.as_mut_ptr().is_null() {
            None
        } else {
            Some(Self { resource })
        }
    }

    pub fn leak(self) -> T {
        self.resource
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.resource.as_mut_ptr().cast()
    }

    pub fn as_mut_ptr(&self) -> *mut c_void {
        self.resource.as_mut_ptr()
    }
}

pub struct ArcKernelObj<T: KernelResource> {
    obj: NonNullKrnlResource<T>,
}

unsafe impl<T: KernelResource> Send for ArcKernelObj<T> {}
unsafe impl<T: KernelResource> Sync for ArcKernelObj<T> {}
unsafe impl<T: KernelResource> DispatchSafe for ArcKernelObj<T> {}

impl<T> ArcKernelObj<T>
where
    T: KernelResource,
{
    pub fn new(obj: NonNullKrnlResource<T>, reference: bool) -> Self {
        unsafe {
            if reference {
                ObfReferenceObject(obj.as_ptr().cast());
            }

            Self { obj }
        }
    }

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

    ///
    /// # Safety
    ///
    /// Must be a valid handle and access rights
    ///
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

            let non_null: Option<NonNullKrnlResource<T>> =
                NonNullKrnlResource::new(T::from_raw_ptr(obj_ptr));

            if let Some(obj) = non_null {
                NtResult::from_status(status, || Self { obj })
            } else {
                Err(NtStatusError::Status(STATUS_OBJECT_TYPE_MISMATCH))
            }
        }
    }

    ///
    /// # Safety
    ///
    /// Must be a valid handle and access rights
    ///
    pub unsafe fn from_raw_user_handle(raw_handle: HANDLE, access: u32) -> NtResult<Self> {
        unsafe {
            let mut obj_ptr = core::ptr::null_mut();

            let status = ObReferenceObjectByHandle(
                raw_handle,
                access,
                T::object_type().into_kernel_object_type(),
                UserMode as _,
                &mut obj_ptr,
                core::ptr::null_mut(),
            );

            let non_null: Option<NonNullKrnlResource<T>> =
                NonNullKrnlResource::new(T::from_raw_ptr(obj_ptr));

            if let Some(obj) = non_null {
                NtResult::from_status(status, || Self { obj })
            } else {
                Err(NtStatusError::Status(STATUS_OBJECT_TYPE_MISMATCH))
            }
        }
    }

    pub unsafe fn as_raw_obj(&self) -> T {
        T::from_raw_ptr(self.obj.as_mut_ptr())
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

            fn as_mut_ptr(&self) -> *mut c_void {
                self.cast()
            }

            unsafe fn from_raw_ptr(ptr: *mut c_void) -> $o {
                ptr.cast()
            }
        }
    };
}

impl_kernel_resource!(PKEVENT, KernelObjectType::Event);
impl_kernel_resource!(PKSEMAPHORE, KernelObjectType::Semaphore);
impl_kernel_resource!(PFILE_OBJECT, KernelObjectType::File);

impl_kernel_resource!(PKPROCESS, KernelObjectType::Process);
impl_kernel_resource!(PKTHREAD, KernelObjectType::Thread);

impl_kernel_resource!(PPROCESS_ACCESS_TOKEN, KernelObjectType::Token);

impl_kernel_resource!(PKENLISTMENT, KernelObjectType::Enlistment);

impl_kernel_resource!(PRKRESOURCEMANAGER, KernelObjectType::ResourceManager);
impl_kernel_resource!(PKTM, KernelObjectType::TranscationManager);
impl_kernel_resource!(PKTRANSACTION, KernelObjectType::Transcation);

unsafe impl WaitableObject for ArcKernelObj<PKTHREAD> {
    fn kernel_object(&self) -> &crate::sys::WaitableKernelObject {
        unsafe { &*(self.obj.as_mut_ptr() as *const crate::sys::WaitableKernelObject) }
    }
}

unsafe impl WaitableObject for ArcKernelObj<PKPROCESS> {
    fn kernel_object(&self) -> &crate::sys::WaitableKernelObject {
        unsafe { &*(self.obj.as_mut_ptr() as *const crate::sys::WaitableKernelObject) }
    }
}
