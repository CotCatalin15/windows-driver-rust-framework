use core::{num::NonZeroU32, ptr::NonNull};

use nt_string::unicode_string::NtUnicodeStr;
use wdk_sys::{
    fltmgr::{FltCloseClientPort, PFLT_PORT, _FLT_FILTER, _FLT_PORT},
    NTSTATUS, STATUS_SUCCESS,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
};

use crate::minifilter::FltFilter;

use super::{FltPort, FltPortCommunicationBuilder};

pub struct FltClient {
    filter: NonNull<_FLT_FILTER>,
    client: NonNull<_FLT_PORT>,
    cookie: Option<()>,
}

impl Drop for FltClient {
    fn drop(&mut self) {
        unsafe {
            FltCloseClientPort(self.filter.as_ptr(), self.client.as_ptr());
        }
    }
}

enum FltCommunicationStorage {
    SingleClient(Option<Arc<FltClients>>),
    MultiClient(),
}

struct FltClientCommunicationInner {
    communication: FltPort,
    storage: FltCommunicationStorage,
}

impl TaggedObject for FltClientCommunicationInner {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fcci")
    }
}

pub struct FltClientCommunication {
    server_cookie: Box<FltClientCommunicationInner>,
}

unsafe impl Sync for FltClientCommunication {}
unsafe impl Send for FltClientCommunication {}

impl FltClientCommunication {
    pub fn new(
        filter: Arc<FltFilter>,
        name: NtUnicodeStr,
        max_connections: NonZeroU32,
    ) -> anyhow::Result<Self> {
        let port = FltPortCommunicationBuilder::new(filter, name)
            .connect(Some(flt_comm_connection))
            .disconnect(Some(flt_comm_disconnect))
            .message(Some(flt_comm_notify))
            .max_connections(max_connections)
            .build()?;

        let storage = match max_connections.get() {
            1 => FltCommunicationStorage::SingleClient(None),
            _ => FltCommunicationStorage::MultiClient(),
        };

        let inner = FltClientCommunicationInner {
            communication: port,
            storage,
        };

        Box::try_create(inner).map(|inner| Self {
            server_cookie: inner,
        })
    }
}

unsafe extern "C" fn flt_comm_connection(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    dbg_break();

    let server_cookie: *mut FltClientCommunicationInner = server_cookie.cast();
    let server_cookie: &mut FltClientCommunicationInner = server_cookie;

    let client = FltClient {
        filter: server_cookie.communication.filter.as_ptr(),
        client: client_port,
        cookie: None,
    };

    STATUS_SUCCESS
}

unsafe extern "C" fn flt_comm_disconnect(client_cookie: *mut core::ffi::c_void) {
    dbg_break();
}

unsafe extern "C" fn flt_comm_notify(
    client_cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    dbg_break();

    STATUS_SUCCESS
}
