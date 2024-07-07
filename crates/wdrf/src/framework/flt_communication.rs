use core::{num::NonZeroU32, ptr::NonNull};

use wdk::nt_success;
use wdk_sys::{
    fltmgr::{
        FltCloseClientPort, FltCloseCommunicationPort, FltCreateCommunicationPort,
        PFLT_CONNECT_NOTIFY, PFLT_DISCONNECT_NOTIFY, PFLT_MESSAGE_NOTIFY, PFLT_PORT, _FLT_FILTER,
        _FLT_PORT,
    },
    NTSTATUS, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    slice::{
        slice_from_raw_parts_mut_or_empty, slice_from_raw_parts_or_empty,
        tracked_slice::TrackedSlice,
    },
    string::ntunicode::NtUnicode,
};

use crate::object::{ObjectAttribs, SecurityDescriptor};

pub struct FltPort(NonNull<_FLT_PORT>);

impl FltPort {
    pub fn try_create(
        filter: NonNull<_FLT_FILTER>,
        name: &mut NtUnicode<'static>,
        max_connections: NonZeroU32,
        cookie: NonNull<u8>,
        connect: PFLT_CONNECT_NOTIFY,
        disconnect: PFLT_DISCONNECT_NOTIFY,
        notify: PFLT_MESSAGE_NOTIFY,
    ) -> anyhow::Result<FltPort> {
        let mut port = core::ptr::null_mut();

        let security_desc = SecurityDescriptor::try_default_flt()?;
        let mut object_attr = ObjectAttribs::new(
            name,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            &security_desc,
        );

        let status = unsafe {
            FltCreateCommunicationPort(
                filter.as_ptr(),
                &mut port,
                object_attr.as_ptr_mut() as _,
                cookie.as_ptr() as _,
                connect,
                disconnect,
                notify,
                max_connections.get() as _,
            )
        };
        if nt_success(status) {
            let port = NonNull::new(port).unwrap();
            Ok(FltPort(port))
        } else {
            Err(anyhow::Error::msg("test"))
        }
    }
}

impl Drop for FltPort {
    fn drop(&mut self) {
        unsafe { FltCloseCommunicationPort(self.0.as_ptr()) }
    }
}

pub struct FltClient(NonNull<_FLT_FILTER>, NonNull<_FLT_PORT>);

impl FltClient {
    pub fn new(filter: NonNull<_FLT_FILTER>, client: NonNull<_FLT_PORT>) -> Self {
        Self(filter, client)
    }
}

impl Drop for FltClient {
    fn drop(&mut self) {
        unsafe {
            FltCloseClientPort(self.0.as_ptr(), &mut self.1.as_ptr());
        }
    }
}

pub trait FltCommunicationDispatch {
    fn on_connect(&mut self, client_port: &mut FltClient, context: &[u8]) -> anyhow::Result<()>;
    fn on_disconnect(&mut self, client: FltClient);

    fn on_notify(&mut self, input: &[u8], output: &mut TrackedSlice) -> anyhow::Result<()>;
}

struct FltCookie<T: FltCommunicationDispatch> {
    filter: NonNull<_FLT_FILTER>,
    client: Option<FltClient>,
    dispatch: T,
}

impl<T> TaggedObject for FltCookie<T> where T: FltCommunicationDispatch {}

pub struct FltSingleClientCommunication<T: FltCommunicationDispatch> {
    port: FltPort,
    cookie: Box<FltCookie<T>>,
}

impl<T> FltSingleClientCommunication<T>
where
    T: FltCommunicationDispatch + 'static + Send + Sync,
{
    pub fn try_create(
        filter: NonNull<_FLT_FILTER>,
        name: &mut NtUnicode<'static>,
        dispatch: T,
    ) -> anyhow::Result<Self> {
        let mut cookie = Box::try_create(FltCookie {
            filter,
            client: None,
            dispatch,
        })?;

        let ptr: *mut FltCookie<T> = cookie.as_mut();
        let port = FltPort::try_create(
            filter,
            name,
            NonZeroU32::new(1).unwrap(),
            NonNull::new(ptr).unwrap().cast(),
            Some(flt_comm_connection::<T>),
            Some(flt_comm_disconnect::<T>),
            Some(flt_comm_notify::<T>),
        )?;

        Ok(Self { port, cookie })
    }
}

unsafe extern "C" fn flt_comm_connection<T: FltCommunicationDispatch>(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let cookie_ptr: *mut FltCookie<T> = server_cookie as _;
    let cookie = &mut *cookie_ptr;

    let mut client = FltClient::new(cookie.filter, NonNull::new(client_port).unwrap());

    let result = cookie.dispatch.on_connect(
        &mut client,
        slice_from_raw_parts_or_empty(connection_context as _, size_of_context as _),
    );

    match result {
        Ok(_) => {
            *connection_port_cookie = cookie_ptr as _;
            cookie.client = Some(client);
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

unsafe extern "C" fn flt_comm_disconnect<T: FltCommunicationDispatch>(
    client_cookie: *mut core::ffi::c_void,
) {
    let cookie: *mut FltCookie<T> = client_cookie as _;
    let cookie = &mut *cookie;

    let client = cookie.client.take().unwrap();
    cookie.dispatch.on_disconnect(client);
}

unsafe extern "C" fn flt_comm_notify<T: FltCommunicationDispatch>(
    client_cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    let cookie: *mut FltCookie<T> = client_cookie as _;
    let cookie = &mut *cookie;

    let mut tracked = TrackedSlice::new(slice_from_raw_parts_mut_or_empty(
        output_buffer as _,
        output_buffer_length as _,
    ));
    let result = cookie.dispatch.on_notify(
        slice_from_raw_parts_or_empty(input_buffer as _, input_buffer_length as _),
        &mut tracked,
    );

    match result {
        Ok(_) => {
            *return_output_buffer_length = tracked.bytes_written() as _;
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}
