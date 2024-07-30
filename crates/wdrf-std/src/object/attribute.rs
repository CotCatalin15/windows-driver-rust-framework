use core::cell::UnsafeCell;

use nt_string::unicode_string::NtUnicodeStr;
use windows_sys::{Wdk::Foundation::OBJECT_ATTRIBUTES, Win32::Security::PSECURITY_DESCRIPTOR};

#[allow(dead_code)]
pub struct ObjectAttributes<'a> {
    inner: UnsafeCell<OBJECT_ATTRIBUTES>,
    name: Option<&'a NtUnicodeStr<'a>>,
    security_descriptor: Option<&'a PSECURITY_DESCRIPTOR>,
}

impl<'a> ObjectAttributes<'a> {
    pub const fn new(attribs: i32) -> Self {
        Self {
            inner: UnsafeCell::new(OBJECT_ATTRIBUTES {
                Attributes: attribs as _,
                ..Self::zeroed_obj_attribs()
            }),
            name: None,
            security_descriptor: None,
        }
    }

    pub fn new_named(name: &'a NtUnicodeStr<'a>, attribs: i32) -> Self {
        Self {
            inner: UnsafeCell::new(OBJECT_ATTRIBUTES {
                Attributes: attribs as _,
                ObjectName: name.as_ptr() as _,
                ..Self::zeroed_obj_attribs()
            }),
            name: Some(name),
            security_descriptor: None,
        }
    }

    pub fn new_named_security(
        name: &'a NtUnicodeStr<'a>,
        attribs: i32,
        descriptor: &'a impl AsRef<PSECURITY_DESCRIPTOR>,
    ) -> Self {
        Self {
            inner: UnsafeCell::new(OBJECT_ATTRIBUTES {
                Attributes: attribs as _,
                ObjectName: name.as_ptr() as _,
                SecurityDescriptor: *descriptor.as_ref(),
                ..Self::zeroed_obj_attribs()
            }),
            name: Some(name),
            security_descriptor: Some(descriptor.as_ref()),
        }
    }

    #[allow(clippy::mut_from_ref)]
    pub fn as_ref_mut(&'a self) -> &'a mut OBJECT_ATTRIBUTES {
        unsafe { &mut *self.inner.get() }
    }

    const fn zeroed_obj_attribs() -> OBJECT_ATTRIBUTES {
        OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<OBJECT_ATTRIBUTES>() as _,
            RootDirectory: 0,
            ObjectName: core::ptr::null_mut(),
            Attributes: 0,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        }
    }
}
