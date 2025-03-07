use core::marker::PhantomData;

use wdrf_std::{
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag},
    vec::Vec,
    NtResult, NtStatusError,
};
use windows_sys::{
    Wdk::Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION,
    Win32::Foundation::STATUS_NO_MEMORY,
};

use crate::minifilter::filter::{
    flt_op_callbacks::{generic_post_op_callback, generic_pre_op_callback},
    registration::FltOperationEntry,
    FltPostOpCallback, FltPreOpCallback,
};

use super::IntoFltOpRegistrationFactory;
use crate::minifilter::structs::IRP_MJ_OPERATION_END;

pub struct MinifilterOperationBuilder<C> {
    registration: Vec<FLT_OPERATION_REGISTRATION>,
    _data: PhantomData<C>,
}

impl<C> MinifilterOperationBuilder<C>
where
    C: 'static + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            registration: Vec::new_in(GlobalKernelAllocator::new(
                MemoryTag::new_from_bytes(b"opre"),
                PoolFlags::POOL_FLAG_NON_PAGED,
            )),
            _data: PhantomData,
        }
    }

    pub fn preop<'a, Pre>(mut self, _preop: Pre, entries: &[FltOperationEntry]) -> NtResult<Self>
    where
        Pre: FltPreOpCallback<'a, MinifilterContext = C, PostContext = ()>,
    {
        self.registration
            .try_reserve(entries.len())
            .map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

        for entry in entries {
            self.registration.push(FLT_OPERATION_REGISTRATION {
                PreOperation: Some(generic_pre_op_callback::<'a, Pre>),
                PostOperation: None,
                ..entry.convert_to_registry()
            });
        }

        Ok(self)
    }

    pub fn operation_with_postop<'a, Post>(
        mut self,
        _postop: Post,
        entries: &[FltOperationEntry],
    ) -> NtResult<Self>
    where
        Post: FltPostOpCallback<'a, MinifilterContext = C>,
    {
        self.registration
            .try_reserve(entries.len())
            .map_err(|_| NtStatusError::Status(STATUS_NO_MEMORY))?;

        for entry in entries {
            self.registration.push(FLT_OPERATION_REGISTRATION {
                PreOperation: Some(generic_pre_op_callback::<'a, Post>),
                PostOperation: Some(generic_post_op_callback::<'a, Post>),
                ..entry.convert_to_registry()
            });
        }

        Ok(self)
    }

    pub fn build(self) -> NtResult<Self> {
        Ok(self)
    }
}

impl<C> IntoFltOpRegistrationFactory for MinifilterOperationBuilder<C>
where
    C: 'static + Send + Sync,
{
    type MinifilterContext = C;

    fn into_operations(
        &mut self,
    ) -> &[windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION] {
        if !self
            .registration
            .last()
            .is_some_and(|last| last.MajorFunction == (IRP_MJ_OPERATION_END as u8))
        {
            self.registration.push({
                FLT_OPERATION_REGISTRATION {
                    MajorFunction: IRP_MJ_OPERATION_END as _,
                    Flags: 0,
                    PreOperation: None,
                    PostOperation: None,
                    Reserved1: core::ptr::null_mut(),
                }
            });
        }

        &self.registration
    }
}
