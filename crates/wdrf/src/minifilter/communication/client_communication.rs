use core::{cell::Cell, num::NonZeroU32, ptr::NonNull, time::Duration};

use nt_string::unicode_string::NtUnicodeStr;

use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::{GlobalKernelAllocator, TaggedObject},
    slice::{
        slice_from_raw_parts_mut_or_empty, slice_from_raw_parts_or_empty,
        tracked_slice::TrackedSlice,
    },
    sync::arc::{Arc, ArcExt},
    NtResult, NtResultEx, NtStatusError,
};
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FltCloseClientPort, FltSendMessage, PFLT_FILTER, PFLT_PORT,
    },
    Win32::Foundation::{
        NTSTATUS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_SUCCESS,
        STATUS_UNSUCCESSFUL,
    },
};

use crate::minifilter::FltFilter;

use super::{FltPort, FltPortCommunicationBuilder};

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
        client: &FltClient<Self::ClientCookie>,
        input: &[u8],
        output: &mut TrackedSlice,
    ) -> anyhow::Result<()>;
}

pub struct FltClient<Cookie: 'static + Send> {
    filter: PFLT_FILTER,
    client: PFLT_PORT,
    cookie: Cell<Option<Cookie>>,
    server_cookie: NonNull<u8>,

    owned: Cell<bool>,
}

impl<Cookie: 'static + Send> FltClient<Cookie> {
    ///
    /// # Safety
    ///
    /// Safe as long as you dont do something bad :)
    ///
    pub unsafe fn as_handle(&self) -> PFLT_PORT {
        self.client
    }

    pub fn get_cookie(&self) -> Option<&Cookie> {
        unsafe { (*self.cookie.as_ptr()).as_ref() }
    }

    pub fn send_with_reply(&self, input: &[u8], output: Option<&mut [u8]>) -> NtResult<u32> {
        unsafe {
            let mut reply_size = output.as_ref().map_or_else(|| 0, |buff| buff.len()) as _;

            let status = FltSendMessage(
                self.filter,
                &self.client,
                input.as_ptr() as _,
                input.len() as _,
                output.map_or_else(core::ptr::null_mut, |buff| buff.as_ptr() as _),
                &mut reply_size,
                core::ptr::null_mut(),
            );

            NtResult::from_status(status, || reply_size)
        }
    }

    pub fn send_with_reply_timeout(
        &self,
        input: &[u8],
        output: Option<&mut [u8]>,
        duration: Duration,
    ) -> NtResult<u32> {
        unsafe {
            let client = self.client;

            let timeout = -((duration.as_nanos() / 100) as i64);

            let mut reply_size = output.as_ref().map_or_else(|| 0, |buff| buff.len()) as _;

            let status = FltSendMessage(
                self.filter,
                &client,
                input.as_ptr() as _,
                input.len() as _,
                output.map_or_else(core::ptr::null_mut, |buff| buff.as_ptr() as _),
                &mut reply_size,
                &timeout,
            );

            NtResult::from_status(status, || reply_size)
        }
    }
}

impl<Cookie: 'static + Send> Drop for FltClient<Cookie> {
    fn drop(&mut self) {
        unsafe {
            if self.owned.get() {
                let mut client_ptr = self.client;
                FltCloseClientPort(self.filter, &mut client_ptr);
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

#[allow(dead_code)]
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
    ) -> NtResult<Self> {
        let storage = match max_connections.get() {
            1 => FltCommunicationStorage::SingleClient(None),
            _ => FltCommunicationStorage::MultiClient(),
        };

        let inner = FltClientCommunicationInner {
            callbacks,
            communication: None,
            storage,
        };

        let mut cookie =
            Box::try_create(inner).map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

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

unsafe extern "system" fn flt_comm_connection<CB: FltCommunicationCallback>(
    client_port: PFLT_PORT,
    server_cookie: *const core::ffi::c_void,
    connection_context: *const core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let server_cookie_ptr: *mut FltClientCommunicationInner<CB> = server_cookie as _;
    let server_cookie = &mut *server_cookie_ptr;

    let communication = server_cookie.communication.as_ref().unwrap();

    let client: FltClient<CB::ClientCookie> = FltClient {
        filter: communication.filter.as_handle(),
        client: client_port,
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

    let cookie = server_cookie.callbacks.on_connect(&client, buffer);

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

unsafe extern "system" fn flt_comm_disconnect<CB: FltCommunicationCallback>(
    client_cookie: *const core::ffi::c_void,
) {
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

unsafe extern "system" fn flt_comm_notify<CB: FltCommunicationCallback>(
    client_cookie: *const core::ffi::c_void,
    input_buffer: *const core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
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

    match server_cookie
        .callbacks
        .on_message(client_cookie, input, &mut tracked_output)
    {
        Ok(_) => {
            *return_output_buffer_length = tracked_output.bytes_written() as _;
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}
