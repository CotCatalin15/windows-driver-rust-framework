use core::{
    ptr::addr_of,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::fields::{Event, Metadata};

pub enum FilterResult {
    Discard,
    Allow,
}

pub trait EventConsumer: 'static + Sync + Send {
    //Called to check if the log should be enabled
    fn enabled(&self) -> bool;

    //Called when stop is called on the log
    fn disable(&self);

    //Returns true if the result should be filtered out
    //Returns false if the result should continue
    fn filter(&self, meta: &Metadata) -> FilterResult;

    fn event(&self, event: &Event);
}

pub struct NullEventConsumer {}

impl EventConsumer for NullEventConsumer {
    fn enabled(&self) -> bool {
        false
    }

    fn disable(&self) {}

    fn filter(&self, _meta: &Metadata) -> FilterResult {
        FilterResult::Discard
    }

    fn event(&self, _event: &Event) {}
}

pub struct ConsumerRegistry {
    consumer: &'static dyn EventConsumer,
    enabled: AtomicBool,
}

unsafe impl Send for ConsumerRegistry {}
unsafe impl Sync for ConsumerRegistry {}

impl ConsumerRegistry {
    pub const fn new(consumer: &'static dyn EventConsumer) -> Self {
        Self {
            consumer,
            enabled: AtomicBool::new(false),
        }
    }

    #[inline]
    pub fn consumer(&self) -> &'static dyn EventConsumer {
        self.consumer
    }

    pub fn enable_consumer(&self) -> bool {
        let enabled = self.consumer.enabled();
        self.enabled.store(enabled, Ordering::SeqCst);

        enabled
    }

    pub fn disable_consumer(&self) {
        self.enabled.store(false, Ordering::SeqCst);
        self.consumer.disable();
    }

    #[inline]
    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn should_log_event(&self, meta: &Metadata) -> bool {
        match self.consumer.filter(meta) {
            FilterResult::Allow => true,
            FilterResult::Discard => false,
        }
    }
}

#[inline]
pub fn get_global_registry() -> &'static ConsumerRegistry {
    unsafe { &*addr_of!(GLOBAL_REGISTRY) }
}

#[inline]
pub fn set_global_consumer(consumer: &'static dyn EventConsumer) -> bool {
    unsafe {
        GLOBAL_REGISTRY = ConsumerRegistry::new(consumer);
    }
    get_global_registry().enable_consumer()
}

static NULL_CONSUMER: NullEventConsumer = NullEventConsumer {};
static mut GLOBAL_REGISTRY: ConsumerRegistry = ConsumerRegistry::new(&NULL_CONSUMER);
