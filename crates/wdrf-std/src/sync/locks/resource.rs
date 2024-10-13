use core::cell::UnsafeCell;

use windows_sys::{
    Wdk::{
        Foundation::ERESOURCE,
        System::SystemServices::{
            ExAcquireResourceExclusiveLite, ExAcquireResourceSharedLite, ExInitializeResourceLite,
            ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion,
        },
    },
    Win32::Foundation::STATUS_NO_MEMORY,
};

use crate::{
    boxed::{Box, BoxExt},
    constants::PoolFlags,
    kmalloc::{MemoryTag, TaggedObject},
    NtResult, NtResultEx, NtStatusError,
};

use super::{MutexGuard, ReadMutexGuard, Unlockable};

pub struct EResource<T> {
    inner: Box<EResourceInner<T>>,
}

unsafe impl<T: Send> Send for EResource<T> {}
unsafe impl<T: Sync> Sync for EResource<T> {}

struct EResourceInner<T> {
    resource: UnsafeCell<ERESOURCE>,
    data: UnsafeCell<T>,
}

impl<T> EResource<T> {
    pub fn try_create(data: T) -> NtResult<Self> {
        let inner = Box::try_create(EResourceInner {
            resource: unsafe { core::mem::zeroed() },
            data: UnsafeCell::new(data),
        })
        .map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;
        let status = unsafe { ExInitializeResourceLite(inner.resource.get()) };

        NtResult::from_status(status, move || Self { inner })
    }

    pub fn write<'a>(&'a self) -> MutexGuard<'a, EResourceUnlockable<'a, T>> {
        unsafe {
            KeEnterCriticalRegion();
            let _ = ExAcquireResourceExclusiveLite(self.inner.resource.get(), true as _);
        }

        MutexGuard::new(
            EResourceUnlockable {
                resource: &self.inner,
            },
            unsafe { &mut *self.inner.data.get() },
        )
    }

    pub fn read<'a>(&'a self) -> ReadMutexGuard<'a, EResourceUnlockable<'a, T>> {
        unsafe {
            KeEnterCriticalRegion();
            let _ = ExAcquireResourceSharedLite(self.inner.resource.get(), true as _);
        }

        ReadMutexGuard::new(
            EResourceUnlockable {
                resource: &self.inner,
            },
            unsafe { &*self.inner.data.get() },
        )
    }
}

impl<T> TaggedObject for EResourceInner<T> {
    fn tag() -> crate::kmalloc::MemoryTag {
        MemoryTag::new_from_bytes(b"eres")
    }

    fn flags() -> crate::constants::PoolFlags {
        PoolFlags::POOL_FLAG_NON_PAGED
    }
}

pub struct EResourceUnlockable<'a, T> {
    resource: &'a EResourceInner<T>,
}

unsafe impl<'a, T> Send for EResourceUnlockable<'a, T> where T: Send {}

impl<'a, T> Unlockable for EResourceUnlockable<'a, T> {
    type Item = T;

    fn unlock(&self) {
        unsafe {
            ExReleaseResourceLite(self.resource.resource.get());
            KeLeaveCriticalRegion();
        }
    }
}
