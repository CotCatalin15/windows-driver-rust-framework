#![no_std]

pub mod consumer;
pub mod fields;
pub mod macros;

#[cfg(test)]
mod tests {
    extern crate std;
    use std::boxed::Box;

    use crate::{
        consumer::{set_global_consumer, EventConsumer},
        event,
        fields::{self, Level},
        info, warn,
    };

    struct MyConsumer {}

    impl EventConsumer for MyConsumer {
        fn enabled(&self) -> bool {
            true
        }

        fn filter(&self, meta: &fields::Metadata) -> bool {
            if meta.level == Level::Info {
                false
            } else {
                true
            }
        }

        fn event(&self, event: &fields::Event) {
            std::println!("Received event: {:#?}", event);
        }
    }

    #[test]
    pub fn test_logs() {
        let consumer = Box::new(MyConsumer {});
        let consumer = Box::leak(consumer);

        set_global_consumer(consumer);

        let a = 11;
        info!("Test");
        warn!(name = "My name", "Test");
    }
}
