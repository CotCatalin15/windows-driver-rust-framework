mod structs;

pub use structs::*;

pub mod collector;
pub mod process_create_notifier;

#[derive(Debug)]
pub enum ProcessCollectorError {
    NoMemory,
    ContextRegisterError,
}
