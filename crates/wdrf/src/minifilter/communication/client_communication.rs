//Single client communication

use core::{num::NonZeroU32, ops::Deref, ptr::NonNull};

use nt_string::unicode_string::NtUnicodeStr;
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    slice::{
        slice_from_raw_parts_mut_or_empty, slice_from_raw_parts_or_empty,
        tracked_slice::TrackedSlice,
    },
    time::Timeout,
    NtResult, NtResultEx, NtStatusError,
};
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::{
        FltCloseClientPort, FltSendMessage, PFLT_FILTER, PFLT_PORT,
    },
    Win32::Foundation::{NTSTATUS, STATUS_NO_MEMORY, STATUS_SUCCESS, STATUS_UNSUCCESSFUL},
};

use crate::minifilter::filter::framework::GLOBAL_MINIFILTER;

use super::{FltPort, FltPortCommunicationBuilder};

pub trait FltCommunicationCallback {
    fn connect(&self, buffer: Option<&[u8]>) -> anyhow::Result<()>;
    fn message(&self, input: &[u8], output: Option<&mut TrackedSlice>) -> anyhow::Result<()>;
    fn disconnect(&self);
}

pub struct FltClientCommunication<CB: FltCommunicationCallback> {
    inner: Box<CommunicationInner<CB>>,
}

pub struct CommunicationInner<CB: FltCommunicationCallback> {
    port: Option<FltPort>,
    callbacks: CB,
    client: FltClient,
}

pub struct FltClient {
    filter: PFLT_FILTER,
    port: PFLT_PORT,
}

impl FltClient {
    pub fn new() -> Self {
        Self {
            filter: GLOBAL_MINIFILTER.get().raw_filter(),
            port: 0,
        }
    }

    fn connect(&mut self, port: PFLT_PORT) {
        self.port = port;
    }

    fn invalidate(&mut self) {
        self.port = 0;
    }

    fn disconnect(&mut self) {
        if self.port > 0 {
            unsafe {
                FltCloseClientPort(self.filter, &mut self.port);
            }
        }
        self.port = 0;
    }

    pub fn send_message(&self, input: &[u8], timeout: Timeout) -> NtResult<()> {
        unsafe {
            let status = FltSendMessage(
                self.filter,
                &self.port,
                input.as_ptr() as _,
                input.len() as _,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                timeout.as_ptr(),
            );

            NtResult::from_status(status, || {})
        }
    }

    pub fn send_message_with_reply<'a>(
        &self,
        input: &[u8],
        reply: &'a mut [u8],
        timeout: Timeout,
    ) -> NtResult<&'a [u8]> {
        unsafe {
            let mut reply_size: u32 = reply.len() as u32;
            let status = FltSendMessage(
                self.filter,
                &self.port,
                input.as_ptr() as _,
                input.len() as _,
                reply.as_ptr() as _,
                &mut reply_size,
                timeout.as_ptr(),
            );
            let reply_size: usize = reply_size as _;
            NtResult::from_status(status, || (&reply[..reply_size]))
        }
    }
}

impl Drop for FltClient {
    fn drop(&mut self) {
        self.disconnect()
    }
}

impl<CB: FltCommunicationCallback> TaggedObject for CommunicationInner<CB> {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"comi")
    }
}

unsafe impl<CB: FltCommunicationCallback> Send for FltClientCommunication<CB> {}
unsafe impl<CB: FltCommunicationCallback> Sync for FltClientCommunication<CB> {}

impl<CB> FltClientCommunication<CB>
where
    CB: FltCommunicationCallback + 'static + Send + Sync,
{
    pub fn new(callbacks: CB, name: NtUnicodeStr) -> NtResult<Self> {
        let mut inner = Box::try_create(CommunicationInner {
            port: None,
            callbacks,
            client: FltClient::new(),
        })
        .map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

        let cookie = inner.as_mut();
        let cookie: *mut CommunicationInner<CB> = cookie;

        let port = FltPortCommunicationBuilder::new(name)
            .max_connections(NonZeroU32::new(1).unwrap())
            .cookie(NonNull::new(cookie).unwrap().cast())
            .connect(Some(flt_comm_connection::<CB>))
            .disconnect(Some(flt_comm_disconnect::<CB>))
            .message(Some(flt_comm_notify::<CB>))
            .build()?;

        inner.port = Some(port);

        Ok(Self { inner })
    }

    pub fn send_message(&self, input: &[u8], timeout: Timeout) -> NtResult<()> {
        self.inner.client.send_message(input, timeout)
    }

    pub fn send_message_with_reply<'a>(
        &self,
        input: &[u8],
        reply: &'a mut [u8],
        timeout: Timeout,
    ) -> NtResult<&'a [u8]> {
        self.inner
            .client
            .send_message_with_reply(input, reply, timeout)
    }
}

impl<CB> Deref for FltClientCommunication<CB>
where
    CB: FltCommunicationCallback,
{
    type Target = CB;

    fn deref(&self) -> &Self::Target {
        &self.inner.callbacks
    }
}

unsafe extern "system" fn flt_comm_connection<CB: FltCommunicationCallback>(
    client_port: PFLT_PORT,
    server_cookie: *const core::ffi::c_void,
    connection_context: *const core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let cookie: *mut CommunicationInner<CB> = server_cookie as *mut CommunicationInner<CB>;
    let cookie = &mut *cookie;

    cookie.client.connect(client_port);

    let answer = if size_of_context > 0 {
        let slice = core::slice::from_raw_parts(connection_context as _, size_of_context as _);
        cookie.callbacks.connect(Some(slice))
    } else {
        cookie.callbacks.connect(None)
    };

    match answer {
        Ok(_) => {
            *connection_port_cookie = server_cookie as _;
            STATUS_SUCCESS
        }
        Err(_) => {
            //Call invalidate so FltCloseClientPort is not called
            cookie.client.invalidate();
            STATUS_UNSUCCESSFUL
        }
    }
}

unsafe extern "system" fn flt_comm_disconnect<CB: FltCommunicationCallback>(
    client_cookie: *const core::ffi::c_void,
) {
    let cookie: *mut CommunicationInner<CB> = client_cookie as *mut CommunicationInner<CB>;
    let cookie = &mut *cookie;

    cookie.callbacks.disconnect();
    cookie.client.disconnect();
}

unsafe extern "system" fn flt_comm_notify<CB: FltCommunicationCallback>(
    client_cookie: *const core::ffi::c_void,
    input_buffer: *const core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    let cookie: *mut CommunicationInner<CB> = client_cookie as *mut CommunicationInner<CB>;
    let cookie = &mut *cookie;

    let input_slice = slice_from_raw_parts_or_empty(input_buffer as _, input_buffer_length as _);
    let output_slice =
        slice_from_raw_parts_mut_or_empty(output_buffer as _, output_buffer_length as _);

    let mut tracked = TrackedSlice::new(output_slice);
    let answer = if output_buffer_length > 0 {
        cookie.callbacks.message(input_slice, Some(&mut tracked))
    } else {
        cookie.callbacks.message(input_slice, None)
    };

    match answer {
        Ok(_) => {
            *return_output_buffer_length = tracked.bytes_written() as u32;
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}
