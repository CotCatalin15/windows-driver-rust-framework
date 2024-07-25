use core::{any::Any, num::NonZeroU32, ptr::NonNull};

use wdk_sys::fltmgr::{
    FltCloseCommunicationPort, FltCreateCommunicationPort, PFLT_CONNECT_NOTIFY,
    PFLT_DISCONNECT_NOTIFY, PFLT_MESSAGE_NOTIFY, _FLT_PORT,
};
use wdrf_std::{kmalloc::TaggedObject, sync::arc::Arc};

use crate::object::ObjectAttribs;

use super::FltFilter;

pub struct FltCommunication {
    filter: Arc<FltFilter>,
    port: NonNull<_FLT_PORT>,
}

unsafe impl Send for FltCommunication {}
unsafe impl Sync for FltCommunication {}

impl TaggedObject for FltCommunication {
    fn tag() -> wdrf_std::kmalloc::MemoryTag {
        wdrf_std::kmalloc::MemoryTag::new_from_bytes(b"fltc")
    }
}

impl Drop for FltCommunication {
    fn drop(&mut self) {
        unsafe {
            FltCloseCommunicationPort(self.port.as_ptr());
        }
    }
}

pub struct FltCommunicationBuilder<'a> {
    filter: Arc<FltFilter>,
    attribs: &'a ObjectAttribs<'a>,
    cookie: Option<NonNull<dyn Any>>,
    connect: PFLT_CONNECT_NOTIFY,
    disconnect: PFLT_DISCONNECT_NOTIFY,
    message: PFLT_MESSAGE_NOTIFY,
    max_connections: NonZeroU32,
}

impl<'a> FltCommunicationBuilder<'a> {
    pub const fn new(filter: Arc<FltFilter>, attribs: &'a ObjectAttribs) -> Self {
        Self {
            filter,
            attribs,
            cookie: None,
            connect: None,
            disconnect: None,
            message: None,
            max_connections: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }

    pub fn cookie(mut self, cookie: NonNull<dyn Any>) -> Self {
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

    pub fn build(self) -> anyhow::Result<FltCommunication> {
        let mut port = core::ptr::null_mut();

        let status = unsafe {
            FltCreateCommunicationPort(
                self.filter.0.as_ptr(),
                &mut port,
                self.attribs.as_ptr() as _,
                core::ptr::null_mut(),
                self.connect,
                self.disconnect,
                self.message,
                self.max_connections.get() as _,
            )
        };
        if !wdk::nt_success(status) {
            Err(anyhow::Error::msg("Failed to create communication port"))
        } else {
            let port = NonNull::new(port).unwrap();
            Ok(FltCommunication::new(self.filter, port))
        }
    }
}

impl FltCommunication {
    fn new(filter: Arc<FltFilter>, port: NonNull<_FLT_PORT>) -> Self {
        Self { filter, port }
    }
}
