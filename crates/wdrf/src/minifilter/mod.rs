use core::mem::MaybeUninit;

use wdk_sys::{
    fltmgr::{
        FltRegisterFilter, FLT_CONTEXT_END, FLT_REGISTRATION_VERSION, IRP_MJ_OPERATION_END,
        _FLT_CONTEXT_REGISTRATION, _FLT_OPERATION_REGISTRATION, _FLT_REGISTRATION,
    },
    DRIVER_OBJECT,
};

pub struct FltRegistration {
    inner: _FLT_REGISTRATION,
}

pub struct FltContextRegistrationSlice {
    inner: &'static [_FLT_CONTEXT_REGISTRATION],
}

impl FltContextRegistrationSlice {
    pub fn new(
        context: &'static [_FLT_CONTEXT_REGISTRATION],
    ) -> Option<FltContextRegistrationSlice> {
        if context.is_empty() {
            None
        } else {
            if context.last().unwrap().ContextType != FLT_CONTEXT_END as _ {
                None
            } else {
                Some(FltContextRegistrationSlice { inner: context })
            }
        }
    }

    pub fn get(&self) -> &'static [_FLT_CONTEXT_REGISTRATION] {
        self.inner
    }
}

pub struct FltOperationRegistrationSlice {
    inner: &'static [_FLT_OPERATION_REGISTRATION],
}

impl FltOperationRegistrationSlice {
    pub fn new(
        context: &'static [_FLT_OPERATION_REGISTRATION],
    ) -> Option<FltOperationRegistrationSlice> {
        if context.is_empty() {
            None
        } else {
            if context.last().unwrap().MajorFunction != IRP_MJ_OPERATION_END as _ {
                None
            } else {
                Some(FltOperationRegistrationSlice { inner: context })
            }
        }
    }

    pub fn get(&self) -> &'static [_FLT_OPERATION_REGISTRATION] {
        self.inner
    }
}

pub struct FltRegistrationBuilder {
    flags: u32,
    context: Option<FltContextRegistrationSlice>,
    operations: Option<FltOperationRegistrationSlice>,
}

impl FltRegistrationBuilder {
    pub fn new() -> Self {
        Self {
            flags: 0,
            context: None,
            operations: None,
        }
    }

    pub fn contexts(mut self, context: FltContextRegistrationSlice) -> Self {
        self.context = Some(context);
        self
    }

    pub fn operations(mut self, op: FltOperationRegistrationSlice) -> Self {
        self.operations = Some(op);
        self
    }

    pub fn build(self) -> anyhow::Result<FltRegistration> {
        let mut registration: _FLT_REGISTRATION = unsafe { MaybeUninit::zeroed().assume_init() };

        registration.Size = core::mem::size_of::<_FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration = self
            .operations
            .map_or_else(|| core::ptr::null(), |c| c.get().as_ptr());

        registration.ContextRegistration = self
            .context
            .map_or_else(|| core::ptr::null(), |c| c.get().as_ptr());

        Ok(FltRegistration {
            inner: registration,
        })
    }
}
