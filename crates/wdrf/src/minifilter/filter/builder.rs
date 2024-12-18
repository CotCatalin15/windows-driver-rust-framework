use crate::context::ContextRegistry;

use super::{
    flt_op_callbacks::flt_minifilter_unload_implementation,
    framework::{MinifilterFramework, GLOBAL_MINIFILTER},
    registration::FltOperationEntry,
    EmptyFltOperationsVisitor, FilterOperationVisitor, FltPostOpCallback, FltPreOpCallback,
};
use wdrf_std::{
    boxed::{Box, BoxExt},
    kmalloc::TaggedObject,
    vec::{Vec, VecCreate},
};
use windows_sys::Wdk::{
    Foundation::DRIVER_OBJECT,
    Storage::FileSystem::Minifilters::{
        FLT_OPERATION_REGISTRATION, FLT_REGISTRATION, FLT_REGISTRATION_VERSION,
    },
};

pub struct MinifilterFrameworkBuilder<
    'a,
    Pre: FltPreOpCallback,
    Post: FltPostOpCallback = EmptyFltOperationsVisitor,
    FilterV: FilterOperationVisitor = EmptyFltOperationsVisitor,
> {
    pre_visitor: Pre,
    post_visitor: Option<Post>,
    operations: &'a [FltOperationEntry],
    filter_visitor: Option<FilterV>,

    flags: u32,
    set_unload: bool,
}

impl<'a, Pre, Post, FilterV> MinifilterFrameworkBuilder<'a, Pre, Post, FilterV>
where
    Pre: FltPreOpCallback + TaggedObject,
    Post: FltPostOpCallback + TaggedObject,
    FilterV: FilterOperationVisitor + TaggedObject,
{
    pub fn new(pre: Pre) -> Self {
        Self {
            pre_visitor: pre,
            post_visitor: None,
            operations: &[],
            filter_visitor: None,
            flags: 0,
            set_unload: false,
        }
    }

    pub fn post(mut self, post: Post) -> Self {
        self.post_visitor = Some(post);
        self
    }

    pub fn operations(mut self, ops: &'a [FltOperationEntry]) -> Self {
        self.operations = ops;
        self
    }

    pub fn filter(mut self, flt: FilterV, unload: bool) -> Self {
        self.filter_visitor = Some(flt);
        self.set_unload = unload;

        self
    }

    pub fn flags(mut self, flags: u32) -> Self {
        self.flags = flags;

        self
    }

    pub fn build_and_register<R: ContextRegistry>(
        self,
        registry: &'static R,
        driver: *const DRIVER_OBJECT,
    ) -> anyhow::Result<()> {
        let mut registration: FLT_REGISTRATION = unsafe { core::mem::zeroed() };

        let operations = self.create_op_registration_vector()?;

        registration.Size = core::mem::size_of::<FLT_REGISTRATION>() as _;
        registration.Version = FLT_REGISTRATION_VERSION as _;
        registration.Flags = self.flags;

        registration.OperationRegistration = if operations.len() == 0 {
            core::ptr::null()
        } else {
            operations.as_ptr()
        };
        registration.ContextRegistration = core::ptr::null();
        registration.FilterUnloadCallback = self
            .set_unload
            .then_some(flt_minifilter_unload_implementation::<FilterV>);
        registration.InstanceSetupCallback = self.instance_setup;
        /*
        registration.InstanceQueryTeardownCallback = self.instance_query_teardown;
        registration.InstanceTeardownCompleteCallback = self.instance_teardown;
        registration.InstanceTeardownStartCallback = self.instance_teardown_start;
        registration.GenerateFileNameCallback = self.generate_filename;
        registration.NormalizeNameComponentCallback = self.normalize_name;
        registration.NormalizeContextCleanupCallback = self.normalize_context;
        registration.TransactionNotificationCallback = self.transcation_notification;
        registration.NormalizeNameComponentExCallback = self.normalize_name_ex;
        registration.SectionNotificationCallback = self.section_notification;
        */

        let preops = Box::try_create(self.pre_visitor)?;
        let postops: Box<dyn FltPostOpCallback> = if let Some(post) = self.post_visitor {
            Box::try_create(post)?
        } else {
            Box::try_create(EmptyFltOperationsVisitor {})?
        };

        let filterops: Box<dyn FilterOperationVisitor> =
            if let Some(filterops) = self.filter_visitor {
                Box::try_create(filterops)?
            } else {
                Box::try_create(EmptyFltOperationsVisitor {})?
            };

        let mut framework = MinifilterFramework::new(preops, postops, filterops);

        unsafe {
            framework
                .register_filter(driver, registration)
                .map_err(|_| anyhow::Error::msg("Failed to register filter"))?
        }

        GLOBAL_MINIFILTER.init(registry, move || framework).unwrap();
        Ok(())
    }

    fn create_op_registration_vector(&self) -> anyhow::Result<Vec<FLT_OPERATION_REGISTRATION>> {
        let mut ops = Vec::create_any();

        if self.operations.len() == 0 {
            Ok(ops)
        } else {
            ops.try_reserve_exact(self.operations.len() + 1)
                .map_err(|_| {
                    anyhow::Error::msg("Failed to reserve exact for the opetions vector")
                })?;

            self.operations.iter().for_each(|entry| {
                ops.push(unsafe { entry.convert_to_registry::<Pre, Post>() });
            });

            ops.push(unsafe { FltOperationEntry::create_end_entry() });
            Ok(ops)
        }
    }
}
