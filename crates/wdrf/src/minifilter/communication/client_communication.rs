use core::{cell::Cell, num::NonZeroU32, ptr::NonNull};

use nt_string::unicode_string::NtUnicodeStr;
use wdk::dbg_break;
use wdk_sys::{
    fltmgr::{FltCloseClientPort, PFLT_PORT, _FLT_FILTER, _FLT_PORT},
    NTSTATUS, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::{GlobalKernelAllocator, TaggedObject},
    slice::{
        slice_from_raw_parts_mut_or_empty, slice_from_raw_parts_or_empty,
        tracked_slice::TrackedSlice,
    },
    sync::arc::{Arc, ArcExt},
};

use crate::minifilter::FltFilter;

use super::{FltPort, FltPortCommunicationBuilder};

pub struct FltClient<Cookie: 'static + Send> {
    filter: NonNull<_FLT_FILTER>,
    client: NonNull<_FLT_PORT>,
    cookie: Cell<Option<Cookie>>,
    server_cookie: NonNull<u8>,

    owned: Cell<bool>,
}

impl<Cookie: 'static + Send> FltClient<Cookie> {
    pub unsafe fn as_ptr(&self) -> NonNull<_FLT_PORT> {
        self.client
    }

    pub fn get_cookie(&self) -> Option<&Cookie> {
        unsafe { (*self.cookie.as_ptr()).as_ref() }
    }
}

impl<Cookie: 'static + Send> Drop for FltClient<Cookie> {
    fn drop(&mut self) {
        unsafe {
            if self.owned.get() {
                let mut client_ptr = self.client.as_ptr();
                FltCloseClientPort(self.filter.as_ptr(), &mut client_ptr);
            }
        }
    }
}

impl<Cookie: 'static + Send> TaggedObject for FltClient<Cookie> {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fltc")
    }
}

unsafe impl<Cookie: 'static + Send> Send for FltClient<Cookie> {}
unsafe impl<Cookie: 'static + Send> Sync for FltClient<Cookie> {}

enum FltCommunicationStorage<Cookie: 'static + Send> {
    SingleClient(Option<Arc<FltClient<Cookie>>>),
    MultiClient(),
}

pub trait FltCommunicationCallback {
    type ClientCookie: 'static + Send;

    fn on_connect(
        &self,
        client: &Arc<FltClient<Self::ClientCookie>>,
        connection_buf: &[u8],
    ) -> anyhow::Result<Option<Self::ClientCookie>>;
    fn on_disconnect(&self, client: &FltClient<Self::ClientCookie>);
    fn on_message(
        &self,
        cookie: Option<&Self::ClientCookie>,
        input: &[u8],
        output: &mut TrackedSlice,
    ) -> anyhow::Result<()>;
}

struct FltClientCommunicationInner<CB>
where
    CB: FltCommunicationCallback,
{
    callbacks: CB,
    communication: Option<FltPort>,
    storage: FltCommunicationStorage<CB::ClientCookie>,
}

impl<CB: FltCommunicationCallback> TaggedObject for FltClientCommunicationInner<CB> {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fcci")
    }
}

pub struct FltClientCommunication<CB>
where
    CB: FltCommunicationCallback,
{
    server_cookie: Box<FltClientCommunicationInner<CB>>,
}

unsafe impl<CB: FltCommunicationCallback> Sync for FltClientCommunication<CB> {}
unsafe impl<CB: FltCommunicationCallback> Send for FltClientCommunication<CB> {}

impl<CB> FltClientCommunication<CB>
where
    CB: FltCommunicationCallback,
{
    pub fn new(
        callbacks: CB,
        filter: Arc<FltFilter>,
        name: NtUnicodeStr,
        max_connections: NonZeroU32,
    ) -> anyhow::Result<Self> {
        let storage = match max_connections.get() {
            1 => FltCommunicationStorage::SingleClient(None),
            _ => FltCommunicationStorage::MultiClient(),
        };

        let inner = FltClientCommunicationInner {
            callbacks,
            communication: None,
            storage,
        };

        let mut cookie = Box::try_create(inner)?;

        let port = unsafe {
            let cookie_ptr: *mut FltClientCommunicationInner<CB> = cookie.as_mut();

            FltPortCommunicationBuilder::new(filter, name)
                .connect(Some(flt_comm_connection::<CB>))
                .disconnect(Some(flt_comm_disconnect::<CB>))
                .message(Some(flt_comm_notify::<CB>))
                .max_connections(max_connections)
                .cookie(NonNull::new_unchecked(cookie_ptr).cast())
                .build()?
        };

        cookie.as_mut().communication = Some(port);

        Ok(Self {
            server_cookie: cookie,
        })
    }
}

unsafe extern "C" fn flt_comm_connection<CB: FltCommunicationCallback>(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    dbg_break();

    let server_cookie_ptr: *mut FltClientCommunicationInner<CB> = server_cookie.cast();
    let server_cookie = &mut *server_cookie_ptr;

    let communication = server_cookie.communication.as_ref().unwrap();

    let client: FltClient<CB::ClientCookie> = FltClient {
        filter: communication.filter.as_ptr(),
        client: NonNull::new(client_port).unwrap(),
        cookie: Cell::new(None),
        server_cookie: NonNull::new(server_cookie_ptr.cast()).unwrap(),
        owned: Cell::new(false),
    };

    let client = Arc::try_create(client);

    if client.is_err() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let client = client.unwrap();

    let buffer =
        slice_from_raw_parts_or_empty(connection_context as *const u8, size_of_context as _);

    let cookie = server_cookie.callbacks.on_connect(&client, &buffer);

    match cookie {
        Ok(cookie) => {
            client.cookie.set(cookie);
            client.owned.set(true);

            match &mut server_cookie.storage {
                FltCommunicationStorage::SingleClient(client_storage) => {
                    *client_storage = Some(client.clone())
                }
                FltCommunicationStorage::MultiClient() => todo!(),
            }

            let leaked_client = Arc::into_raw(client);
            *connection_port_cookie = leaked_client as _;

            STATUS_SUCCESS
        }
        Err(_) => STATUS_INSUFFICIENT_RESOURCES,
    }
}

unsafe extern "C" fn flt_comm_disconnect<CB: FltCommunicationCallback>(
    client_cookie: *mut core::ffi::c_void,
) {
    dbg_break();

    let client_cookie: *const FltClient<CB::ClientCookie> = client_cookie.cast();

    let client_cookie = Arc::from_raw_in(
        client_cookie,
        GlobalKernelAllocator::new_for_tagged::<FltClient<CB::ClientCookie>>(),
    );

    let server_cookie: *mut FltClientCommunicationInner<CB> =
        client_cookie.server_cookie.as_ptr().cast();

    let server_cookie = &mut *server_cookie;
    server_cookie
        .callbacks
        .on_disconnect(client_cookie.as_ref());

    server_cookie.storage = FltCommunicationStorage::SingleClient(None);
}

unsafe extern "C" fn flt_comm_notify<CB: FltCommunicationCallback>(
    client_cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    dbg_break();
    *return_output_buffer_length = 0;

    let client_cookie: *const FltClient<CB::ClientCookie> = client_cookie.cast();
    let client_cookie = &*client_cookie;

    let server_cookie: *mut FltClientCommunicationInner<CB> =
        client_cookie.server_cookie.as_ptr().cast();

    let server_cookie = &mut *server_cookie;

    let input = slice_from_raw_parts_or_empty(input_buffer as *const u8, input_buffer_length as _);
    let output =
        slice_from_raw_parts_mut_or_empty(output_buffer as *mut u8, output_buffer_length as _);

    let mut tracked_output: TrackedSlice = TrackedSlice::new(output);

    match server_cookie.callbacks.on_message(
        client_cookie.get_cookie(),
        &input,
        &mut tracked_output,
    ) {
        Ok(_) => {
            *return_output_buffer_length = tracked_output.bytes_written() as _;
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}
