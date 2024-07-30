pub mod this_thread;

use core::cell::UnsafeCell;

use windows_sys::{
    Wdk::System::SystemServices::PsCreateSystemThread,
    Win32::{
        Foundation::{HANDLE, STATUS_NO_MEMORY},
        System::{Kernel::OBJ_KERNEL_HANDLE, Threading::THREAD_ALL_ACCESS},
    },
};

use windows_sys::Win32::Storage::FileSystem::{DELETE, SYNCHRONIZE};

use crate::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject},
    nt_success,
    object::{attribute::ObjectAttributes, ArcKernelObj},
    structs::PKTHREAD,
    sync::arc::{Arc, ArcExt},
    sys::{WaitableKernelObject, WaitableObject},
    NtResult, NtStatusError,
};

struct InnerPacket<T> {
    thread_obj: Option<ArcKernelObj<PKTHREAD>>,
    result: Option<T>,
    function: Option<Box<dyn FnOnce() -> T>>,
}

struct Packet<T> {
    inner: UnsafeCell<InnerPacket<T>>,
}

impl<T> Packet<T>
where
    T: 'static + Send,
{
    pub fn new<F>(fnc: F) -> anyhow::Result<Self>
    where
        F: FnOnce() -> T,
        F: 'static + Send,
    {
        let fnc = Box::try_create_in(
            fnc,
            GlobalKernelAllocator::new(
                MemoryTag::new_from_bytes(b"func"),
                PoolFlags::POOL_FLAG_NON_PAGED,
            ),
        )?;

        let inner = InnerPacket {
            thread_obj: None,
            result: None,
            function: Some(fnc),
        };
        Ok(Self {
            inner: UnsafeCell::new(inner),
        })
    }
}

impl<T> TaggedObject for Packet<T>
where
    T: 'static + Send,
{
    fn tag() -> MemoryTag {
        MemoryTag::new_from_bytes(b"pack")
    }
}

pub struct JoinHandle<T> {
    packet: Arc<Packet<T>>,
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        let _ = self.wait();
    }
}

unsafe impl<T> Send for JoinHandle<T> {}
unsafe impl<T> Sync for JoinHandle<T> {}

unsafe impl<T> WaitableObject for JoinHandle<T> {
    fn kernel_object(&self) -> &WaitableKernelObject {
        unsafe {
            let packet = &mut *self.packet.inner.get();
            let ptr: *const PKTHREAD = packet.thread_obj.as_ref().unwrap().raw_obj();

            &*ptr.cast()
        }
    }
}

impl<T> JoinHandle<T> {
    pub fn join(self) -> T {
        self.wait();

        unsafe {
            let packet = &mut *self.packet.inner.get();
            packet.result.take().unwrap()
        }
    }

    pub fn detach(self) {}

    pub fn thread_object(&self) -> &ArcKernelObj<PKTHREAD> {
        unsafe {
            let packet = &mut *self.packet.inner.get();
            packet.thread_obj.as_ref().unwrap_unchecked()
        }
    }
}

pub fn spawn<T, F>(f: F) -> NtResult<JoinHandle<T>>
where
    F: FnOnce() -> T,
    F: 'static + Send,
    T: 'static + Send,
{
    let p = Packet::new(f).map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;
    let packet = Arc::try_create(p).map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

    unsafe {
        let mut handle: HANDLE = 0;

        let leaked = Arc::into_raw(packet.clone());

        let obj = ObjectAttributes::new(OBJ_KERNEL_HANDLE);

        let fnc: unsafe extern "system" fn(*mut core::ffi::c_void) = thread_main::<T>;
        let status = PsCreateSystemThread(
            &mut handle,
            DELETE | SYNCHRONIZE,
            obj.as_ref_mut(),
            0,
            core::ptr::null_mut(),
            Some(core::mem::transmute(fnc)), //
            leaked as _,
        );

        if !nt_success(status) {
            let _ = Arc::from_raw_in(leaked, GlobalKernelAllocator::new_for_tagged::<Packet<T>>());
            Err(NtStatusError::Status(status))
        } else {
            let obj = ArcKernelObj::from_raw_handle(handle, THREAD_ALL_ACCESS)?;
            (*packet.inner.get()).thread_obj = Some(obj);
            Ok(JoinHandle { packet })
        }
    }
}

impl<T> Packet<T> where T: 'static + Send {}

unsafe extern "system" fn thread_main<T: 'static + Send>(context: *mut core::ffi::c_void) {
    let leaked: *const Packet<T> = context as _;

    let packet = Arc::from_raw_in(leaked, GlobalKernelAllocator::new_for_tagged::<Packet<T>>());

    let packet = &mut *packet.inner.get();
    let fnc = packet.function.take().unwrap();

    let result = (fnc)();
    packet.result = Some(result);
}
