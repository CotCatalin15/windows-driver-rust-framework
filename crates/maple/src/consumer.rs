use crate::fields::{Event, Metadata};

pub trait EventConsumer: 'static + Sync + Send {
    fn enabled(&self) -> bool;

    fn filter(&self, _meta: &Metadata) -> bool {
        true
    }

    fn event(&self, event: &Event);
}

pub struct NullEventConsumer {}

impl EventConsumer for NullEventConsumer {
    fn enabled(&self) -> bool {
        false
    }

    fn event(&self, _event: &Event) {}
}

pub fn set_global_consumer(consumer: &'static dyn EventConsumer) {
    unsafe { GLOBAL_CONSUMER = consumer }
}

#[inline]
pub fn get_global() -> &'static dyn EventConsumer {
    unsafe { GLOBAL_CONSUMER }
}

#[inline]
pub fn is_consumer_enabled() -> bool {
    get_global().enabled()
}

#[inline]
pub fn is_filter_pass(meta: &Metadata) -> bool {
    get_global().filter(meta)
}

static NULL_CONSUMER: NullEventConsumer = NullEventConsumer {};
static mut GLOBAL_CONSUMER: &dyn EventConsumer = &NULL_CONSUMER;
