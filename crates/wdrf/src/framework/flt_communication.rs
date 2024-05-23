use core::{any::Any, ffi::c_void, mem::MaybeUninit, num::NonZeroU32, pin::Pin, ptr::NonNull};

use derive_builder::Builder;
use wdk_sys::{
    fltmgr::{
        FltCloseCommunicationPort, FltCreateCommunicationPort, NTSTATUS, PFLT_PORT, _FLT_FILTER,
        _FLT_PORT,
    },
    OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, STATUS_SUCCESS, STATUS_UNSUCCESSFUL,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    string::ntunicode::NtUnicode,
};

use crate::object::{ObjectAttribs, SecurityDescriptor};

pub type FltCommunicationDisconnect = fn(cookie: Option<&mut dyn Any>);
pub type FltCommunicationNotify =
    fn(cookie: Option<&mut dyn Any>, input: &[u8], output: &mut [u8]) -> anyhow::Result<u32>;

pub type FltCommunicationConnect =
    fn(server_cookie: Option<&mut dyn Any>) -> anyhow::Result<Option<Box<dyn Any>>>;

#[derive(Builder)]
#[builder(no_std)]
pub struct FltCommunicationDispatch {
    disconnect: Option<FltCommunicationDisconnect>,
    notify: Option<FltCommunicationNotify>,
}

struct FltCommunicationCookie {
    communication: NonNull<FltCommunication>,
    cookie: Option<Box<dyn Any>>,
    dispatch: FltCommunicationDispatch,
}

impl TaggedObject for FltCommunicationCookie {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"flck")
    }
}

pub struct FltCommunication {
    port: NonNull<_FLT_PORT>,
    cookie: Box<FltCommunicationCookie>,
}

impl Drop for FltCommunication {
    fn drop(&mut self) {
        unsafe {
            FltCloseCommunicationPort(self.port.as_ptr());
        }
    }
}

impl FltCommunication {
    pub(super) fn try_create<C>(
        name: NtUnicode<'static>,
        max_connections: NonZeroU32,
        user_cookie: Option<Box<dyn Any>>,
        dispatch: FltCommunicationDispatch,
    ) -> anyhow::Result<FltCommunication> {
        unsafe {
            let mut port = core::ptr::null_mut();

            let security_desc = SecurityDescriptor::try_default_flt()?;
            let mut object_attr = ObjectAttribs::new(
                name,
                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                &security_desc,
            );

            let mut cookie = Box::try_create(FltCommunicationCookie {
                communication: NonNull::dangling(),
                cookie: user_cookie,
                dispatch,
            })?;

            let pt: *mut FltCommunicationCookie = cookie.as_mut();

            FltCreateCommunicationPort(
                filter.as_ptr(),
                port,
                object_attr.as_ptr_mut() as _,
                pt as _,
                MessageNotifyCallback,
                Some(flt_comm_disconnect),
                Some(flt_comm_notify),
                max_connections.get() as _,
            );
        }

        return Err(anyhow::Error::msg("Failed to create communication"));
    }
}

unsafe extern "C" fn flt_comm_disconnect(cookie: *mut core::ffi::c_void) {
    let cookie: *mut dyn Any = cookie as _;
    let mut cookie = &mut *cookie;

    let cookie = cookie
        .downcast_mut::<FltCommunicationCookie>()
        .unwrap_unchecked();

    if let Some(ref cb) = cookie.dispatch.disconnect {
        let context = cookie.cookie.as_mut().map(|b| b.as_mut());
        cb(context);
    }
}

unsafe extern "C" fn flt_comm_notify(
    cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
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
}

/*
typedef NTSTATUS
(*PFLT_CONNECT_NOTIFY) (
      IN PFLT_PORT ClientPort,
      IN PVOID ServerPortCookie,
      IN PVOID ConnectionContext,
      IN ULONG SizeOfContext,
      OUT PVOID *ConnectionPortCookie
      );
 */
unsafe extern "C" fn flt_comm_connection(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: *mut u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let server_cookie: *mut dyn Any = server_cookie as _;
    let mut cookie = &mut *server_cookie;

    let cookie = cookie
        .downcast_mut::<FltCommunicationCookie>()
        .unwrap_unchecked();

    STATUS_SUCCESS
}
