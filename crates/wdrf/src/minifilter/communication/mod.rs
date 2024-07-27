pub mod client_communication;

use core::{num::NonZeroU32, ptr::NonNull};

use nt_string::unicode_string::NtUnicodeStr;

use wdrf_std::{
    kmalloc::TaggedObject, object::attribute::ObjectAttributes, sync::arc::Arc, NtResult,
    NtResultEx,
};
use windows_sys::{
    Wdk::{
        Foundation::OBJECT_ATTRIBUTES,
        Storage::FileSystem::Minifilters::{
            FltCloseCommunicationPort, FltCreateCommunicationPort, PFLT_CONNECT_NOTIFY,
            PFLT_DISCONNECT_NOTIFY, PFLT_MESSAGE_NOTIFY, PFLT_PORT,
        },
    },
    Win32::System::Kernel::{OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE},
};

use super::{security_descriptor::FltSecurityDescriptor, FltFilter};

pub struct FltPort {
    filter: Arc<FltFilter>,
    port: PFLT_PORT,
    max_clients: u32,
}

unsafe impl Send for FltPort {}
unsafe impl Sync for FltPort {}

impl TaggedObject for FltPort {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fltc")
    }
}

pub struct FltPortCommunicationBuilder<'a> {
    filter: Arc<FltFilter>,
    name: NtUnicodeStr<'a>,
    cookie: Option<NonNull<()>>,
    connect: PFLT_CONNECT_NOTIFY,
    disconnect: PFLT_DISCONNECT_NOTIFY,
    message: PFLT_MESSAGE_NOTIFY,
    max_connections: NonZeroU32,
}

impl<'a> FltPortCommunicationBuilder<'a> {
    pub fn new(filter: Arc<FltFilter>, name: NtUnicodeStr<'a>) -> Self {
        Self {
            filter,
            name,
            cookie: None,
            connect: None,
            disconnect: None,
            message: None,
            max_connections: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }

    pub fn cookie(mut self, cookie: NonNull<()>) -> Self {
        self.cookie = Some(cookie);
        self
    }

    pub fn connect(mut self, connect: PFLT_CONNECT_NOTIFY) -> Self {
        self.connect = connect;
        self
    }

    pub fn disconnect(mut self, disconnect: PFLT_DISCONNECT_NOTIFY) -> Self {
        self.disconnect = disconnect;
        self
    }

    pub fn message(mut self, message: PFLT_MESSAGE_NOTIFY) -> Self {
        self.message = message;
        self
    }

    pub fn max_connections(mut self, max_connections: NonZeroU32) -> Self {
        self.max_connections = max_connections;
        self
    }

    pub fn build(self) -> NtResult<FltPort> {
        let mut port = 0;
        let security_descriptor = FltSecurityDescriptor::try_default_flt()?;
        let obj_attribs = ObjectAttributes::new_named_security(
            &self.name,
            OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
            &security_descriptor,
        );

        let ptr: *mut OBJECT_ATTRIBUTES = obj_attribs.as_ref_mut();

        let cookie = match self.cookie {
            Some(cookie) => cookie.as_ptr().cast(),
            None => core::ptr::null_mut(),
        };
        let status = unsafe {
            FltCreateCommunicationPort(
                self.filter.as_handle(),
                &mut port,
                ptr.cast(),
                cookie,
                self.connect,
                self.disconnect,
                self.message,
                self.max_connections.get() as _,
            )
        };

        NtResult::from_status(status, || {
            FltPort::new(self.filter, port, self.max_connections.get())
        })
    }
}

impl FltPort {
    fn new(filter: Arc<FltFilter>, port: isize, max_clients: u32) -> Self {
        Self {
            filter,
            port,
            max_clients,
        }
    }

    pub fn max_clients(&self) -> u32 {
        self.max_clients
    }
}

impl Drop for FltPort {
    fn drop(&mut self) {
        unsafe {
            FltCloseCommunicationPort(self.port);
        }
    }
}
