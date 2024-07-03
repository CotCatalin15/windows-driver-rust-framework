use core::{any::Any, ffi::c_void, mem::MaybeUninit, num::NonZeroU32, pin::Pin, ptr::NonNull};

use derive_builder::Builder;
use wdk::{nt_success, wdf::SpinLock};
use wdk_sys::{
    fltmgr::{
        FltCloseClientPort, FltCloseCommunicationPort, FltCreateCommunicationPort, NTSTATUS,
        PFLT_FILTER, PFLT_PORT, _FLT_FILTER, _FLT_PORT,
    },
    OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    string::ntunicode::NtUnicode,
    sync::{arc::Arc, rwlock::RwLock},
};

use crate::object::{ObjectAttribs, SecurityDescriptor};

pub trait FltCommunicationDispatch {
    fn on_connect(
        &mut self,
        client_port: PFLT_PORT,
        context: &[u8],
    ) -> anyhow::Result<Option<Box<dyn Any>>>;

    fn on_disconnect(&mut self, cookie: Option<&mut dyn Any>);

    fn on_notify(
        &mut self,
        cookie: Option<&mut dyn Any>,
        input: &[u8],
        output: &mut [u8],
    ) -> anyhow::Result<u32>;
}

struct FltPort(NonNull<_FLT_PORT>);

impl FltPort {
    pub fn new(port: NonNull<_FLT_PORT>) -> Self {
        Self(port)
    }
}

impl Drop for FltPort {
    fn drop(&mut self) {
        unsafe {
            FltCloseCommunicationPort(self.0.as_ptr());
        }
    }
}

struct FltClient(NonNull<_FLT_FILTER>, NonNull<_FLT_PORT>);

impl FltClient {
    pub fn new(filter: NonNull<_FLT_FILTER>, port: NonNull<_FLT_PORT>) -> Self {
        Self(filter, port)
    }
}

impl Drop for FltClient {
    fn drop(&mut self) {
        unsafe {
            FltCloseClientPort(self.0.as_ptr(), self.1.as_ptr());
        }
    }
}

pub struct FltSingleClientCommunication<T: FltCommunicationDispatch> {
    port: FltPort,
    cookie: Box<FltServerCookie<T>>,
}

struct FltServerCookie<T: FltCommunicationDispatch> {
    filter: NonNull<_FLT_FILTER>,
    client: SpinLock,
    dispatch: T,
}

impl<T: FltCommunicationDispatch> TaggedObject for FltServerCookie<T> {}

impl<T> FltSingleClientCommunication<T>
where
    T: FltCommunicationDispatch,
{
    pub(super) fn try_create(
        filter: NonNull<_FLT_FILTER>,
        name: NtUnicode<'static>,
        max_connections: NonZeroU32,
        dispatch: T,
    ) -> anyhow::Result<FltSingleClientCommunication<T>> {
        unsafe {
            let mut port = core::ptr::null_mut();

            let security_desc = SecurityDescriptor::try_default_flt()?;
            let mut object_attr = ObjectAttribs::new(
                name,
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                &security_desc,
            );

            let mut dispatch = Box::try_create(FltServerCookie {
                filter: filter.clone(),
                dispatch: dispatch,
            })?;

            let ptr: *mut FltServerCookie<T> = dispatch.as_mut();
            let status = FltCreateCommunicationPort(
                filter.as_ptr(),
                &mut port,
                object_attr.as_ptr_mut() as _,
                ptr as _,
                Some(flt_comm_connection::<T>),
                Some(flt_comm_disconnect::<T>),
                Some(flt_comm_notify::<T>),
                max_connections.get() as _,
            );
            if !nt_success(status) {
                return Err(anyhow::Error::msg(
                    "FltCreateCommunicationPort failed to create a communication",
                ));
            }

            let port = NonNull::new(port).unwrap();
            let port = FltPort(port);

            return Ok(FltSingleClientCommunication {
                port: port,
                server_cookie: dispatch,
            });
        }
    }
}

unsafe extern "C" fn flt_comm_connection<T: FltCommunicationDispatch>(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let server_cookie = server_cookie.cast::<FltServerCookie<T>>();
    let mut cookie = &mut *server_cookie;

    let result = cookie.dispatch.on_connect(
        client_port,
        core::slice::from_raw_parts(connection_context as _, size_of_context as _),
    );
    match result {
        Ok(context) => {
            if let Some(context) = context {
                let ptr: &'static mut dyn Any = Box::leak(context);
                let raw_ptr: *mut dyn Any = ptr as _;
                (*connection_port_cookie) = raw_ptr as _;
            }
            STATUS_SUCCESS
        }
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

unsafe extern "C" fn flt_comm_disconnect<T: FltCommunicationDispatch>(
    cookie: *mut core::ffi::c_void,
) {
    let cookie: *mut dyn Any = cookie as _;
    let cookie = &mut *cookie;

    let cookie = cookie
        .downcast_mut::<FltServerCookie<T>>()
        .unwrap_unchecked();
}

unsafe extern "C" fn flt_comm_notify<T: FltCommunicationDispatch>(
    cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    /*
    let cookie: *mut dyn Any = cookie as _;
    let mut cookie = &mut *cookie;

    let cookie = cookie
        .downcast_mut::<FltCommunicationCookie>()
        .unwrap_unchecked();

    if let Some(ref cb) = cookie.dispatch.notify {
        let input: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(input_buffer as *mut u8, input_buffer_length as _)
        };

        let mut output: &mut [u8] = unsafe {
            core::slice::from_raw_parts_mut(output_buffer as *mut u8, output_buffer_length as _)
        };

        let context = cookie.cookie.as_mut().map(|b| b.as_mut());
        if let Ok(bytes_written) = cb(context, &input, &mut output) {
            if bytes_written >= output.len() {
                //TODO: replace panic with a log and status error
                panic!("Bytes writen greater than avalible");
            }
            STATUS_SUCCESS
        } else {
            STATUS_UNSUCCESSFUL
        }
    } else {
        STATUS_SUCCESS
    }
    */
    STATUS_SUCCESS
}
