use wdrf_std::{
    constants::PoolFlags,
    kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject},
    vec::Vec,
};
use windows_sys::Wdk::Storage::FileSystem::Minifilters::FLT_OPERATION_REGISTRATION;

use crate::minifilter::filter::{
    flt_op_callbacks::{generic_post_op_callback, generic_pre_op_callback},
    registration::FltOperationEntry,
    FltPostOpCallback, FltPreOpCallback,
};

use super::IntoFltOpRegistrationFactory;
use crate::minifilter::structs::IRP_MJ_OPERATION_END;

pub struct MinifilterOperationBuilder {
    registration: Vec<FLT_OPERATION_REGISTRATION>,
}

impl MinifilterOperationBuilder {
    pub fn new() -> Self {
        Self {
            registration: Vec::new_in(GlobalKernelAllocator::new(
                MemoryTag::new_from_bytes(b"opre"),
                PoolFlags::POOL_FLAG_NON_PAGED,
            )),
        }
    }

    pub fn preop<'a, Pre, C: Sized + 'static + Send + Sync>(
        mut self,
        preop: Pre,
        entries: &[FltOperationEntry],
    ) -> Self
    where
        Pre: FltPreOpCallback<'a, C, ()>,
    {
        for entry in entries {
            self.registration.push(FLT_OPERATION_REGISTRATION {
                PreOperation: Some(generic_pre_op_callback::<'a, Pre, C, ()>),
                PostOperation: None,
                ..entry.convert_to_registry()
            });
        }

        self
    }

    pub fn operation_with_postop<'a, Pre, Post, C: Sized + 'static + Send + Sync, PostContext>(
        mut self,
        _preop: Pre,
        _postop: Post,
        entries: &[FltOperationEntry],
    ) -> Self
    where
        Pre: FltPreOpCallback<'a, C, PostContext>,
        Post: FltPostOpCallback<'a, C, PostContext>,
        PostContext: 'static + Send + Sync + TaggedObject,
    {
        for entry in entries {
            self.registration.push(FLT_OPERATION_REGISTRATION {
                PreOperation: Some(generic_pre_op_callback::<'a, Pre, C, PostContext>),
                PostOperation: Some(generic_post_op_callback::<'a, Post, C, PostContext>),
                ..entry.convert_to_registry()
            });
        }

        self
    }
}

impl IntoFltOpRegistrationFactory for MinifilterOperationBuilder {
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
