#![no_std]

pub mod consumer;
pub mod fields;
pub mod macros;

#[cfg(test)]
mod tests {
    extern crate std;
    use std::boxed::Box;

    use crate::{
        consumer::{set_global_consumer, EventConsumer, FilterResult},
        event,
        fields::{self, Level},
        info, warn,
    };

    struct MyConsumer {}

    impl EventConsumer for MyConsumer {
        fn enabled(&self) -> bool {
            true
        }

        fn disable(&self) {}

        fn filter(&self, meta: &fields::Metadata) -> FilterResult {
            if meta.level == Level::Info {
                FilterResult::Discard
            } else {
                FilterResult::Allow
            }
        }

        fn event(&self, event: &fields::Event) {
            std::println!("Received event: {:#?}", event);
        }
    }

    #[test]
    pub fn test_logs() {
        let r: anyhow::Result<u32> = Err(anyhow::Error::msg("Test"));

        let consumer = Box::new(MyConsumer {});
        let consumer = Box::leak(consumer);

        let _ = set_global_consumer(consumer);

        let a = 11;

        warn!(
            name = "test_logs",
            result = r,
            "This result contains a name"
        );
        warn!(result = r, "This result does not contain a name");
    }
}
