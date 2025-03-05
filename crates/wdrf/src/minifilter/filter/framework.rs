use core::{any::Any, cell::UnsafeCell};

use wdrf_std::kmalloc::TaggedObject;
use wdrf_std::{
    boxed::{Box, BoxExt},
    NtResult, NtResultEx, NtStatusError,
};

use windows_sys::Win32::Foundation::STATUS_NO_MEMORY;

use windows_sys::Wdk::{
    Foundation::DRIVER_OBJECT,
    Storage::FileSystem::Minifilters::{
        FltRegisterFilter, FltStartFiltering, FltUnregisterFilter, FLT_REGISTRATION, PFLT_FILTER,
    },
};

use crate::context::Context;
use wdrf_std::traits::DispatchSafe;

pub struct MinifilterContext<C: ?Sized + 'static + Send + Sync> {
    pub(crate) context: Box<C>,
}

unsafe impl<C: ?Sized + 'static + Send + Sync> Send for MinifilterContext<C> {}
unsafe impl<C: ?Sized + 'static + Send + Sync> Sync for MinifilterContext<C> {}
unsafe impl<C: ?Sized + 'static + Send + Sync + DispatchSafe> DispatchSafe
    for MinifilterContext<C>
{
}

pub type MinifilterContextAny = MinifilterContext<dyn Any + 'static + Send + Sync>;

impl<C> MinifilterContext<C>
where
    C: 'static + Send + Sync + TaggedObject + Any,
{
    pub fn try_create(context: C) -> NtResult<Self> {
        let context =
            Box::try_create(context).map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

        Ok(Self { context })
    }
}

impl<C> MinifilterContext<C>
where
    C: 'static + Send + Sync + Any,
{
    pub(crate) fn into_any(self) -> MinifilterContextAny {
        MinifilterContext {
            context: self.context,
        }
    }
}

impl MinifilterContextAny {
    pub(crate) fn cast_ref_unsafe<C: Send + Sync + 'static>(
        context: &'static MinifilterContextAny,
    ) -> &'static C {
        unsafe { context.context.downcast_ref_unchecked::<C>() }
    }
}

pub struct MinifilterFramework {
    pub(crate) minifilter_context: MinifilterContextAny,
    pub(crate) filter: UnsafeCell<PFLT_FILTER>,
}

unsafe impl Send for MinifilterFramework {}
unsafe impl Sync for MinifilterFramework {}

pub(crate) static GLOBAL_MINIFILTER: Context<MinifilterFramework> = Context::uninit();

impl MinifilterFramework {
    pub(crate) fn new(context: MinifilterContextAny) -> Self {
        Self {
            minifilter_context: context,
            filter: UnsafeCell::new(0),
        }
    }

    pub(crate) unsafe fn register_filter(
        &mut self,
        driver: *const DRIVER_OBJECT,
        registration: FLT_REGISTRATION,
    ) -> NtResult<()> {
        let status = FltRegisterFilter(driver, &registration, self.filter.get());

        NtResult::from_status(status, || ())
    }

    pub unsafe fn start_filtering() -> NtResult<()> {
        let status = FltStartFiltering(*GLOBAL_MINIFILTER.get().filter.get());

        NtResult::from_status(status, || ())
    }

    pub fn unregister() {
        unsafe {
            let framework = GLOBAL_MINIFILTER.get();
            if *framework.filter.get() != 0 {
                FltUnregisterFilter(*framework.filter.get());
                *framework.filter.get() = 0;
            }
        }
    }

    pub fn raw_filter(&self) -> PFLT_FILTER {
        unsafe { *self.filter.get() }
    }

    pub(crate) fn context<C>() -> &'static C
    where
        C: Sized + 'static + Send + Sync,
    {
        let context = unsafe {
            GLOBAL_MINIFILTER
                .get()
                .minifilter_context
                .context
                .downcast_ref_unchecked::<C>()
        };

        context
    }
}

impl Drop for MinifilterFramework {
    fn drop(&mut self) {
        Self::unregister();
    }
}
