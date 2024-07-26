use core::{any::Any, num::NonZeroU32, ptr::NonNull};

use nt_string::unicode_string::NtUnicodeStr;
use wdk_sys::{
    fltmgr::{
        FltCloseCommunicationPort, FltCreateCommunicationPort, PFLT_CONNECT_NOTIFY,
        PFLT_DISCONNECT_NOTIFY, PFLT_MESSAGE_NOTIFY, _FLT_PORT,
    },
    OBJECT_ATTRIBUTES, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE,
};
use wdrf_std::{kmalloc::TaggedObject, object::attribute::ObjectAttributes, sync::arc::Arc};

use super::{security_descriptor::FltSecurityDescriptor, FltFilter};

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

pub struct FltCommunicationBuilder<'a> {
    filter: Arc<FltFilter>,
    name: NtUnicodeStr<'a>,
    cookie: Option<NonNull<dyn Any>>,
    connect: PFLT_CONNECT_NOTIFY,
    disconnect: PFLT_DISCONNECT_NOTIFY,
    message: PFLT_MESSAGE_NOTIFY,
    max_connections: NonZeroU32,
}

impl<'a> FltCommunicationBuilder<'a> {
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
        let security_descriptor = FltSecurityDescriptor::try_default_flt()?;
        let obj_attribs = ObjectAttributes::new_named_security(
            &self.name,
            OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
            &security_descriptor,
        );

        let ptr: *mut OBJECT_ATTRIBUTES = obj_attribs.as_ref_mut();
        let status = unsafe {
            FltCreateCommunicationPort(
                self.filter.0.as_ptr(),
                &mut port,
                ptr.cast(),
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

impl Drop for FltCommunication {
    fn drop(&mut self) {
        unsafe {
            FltCloseCommunicationPort(self.port.as_ptr());
        }
    }
}
