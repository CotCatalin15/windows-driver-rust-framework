mod structs;

pub use structs::*;
use wdrf_std::NtStatusError;

pub mod collector;
pub mod notifier;
pub mod process_create_notifier;

#[derive(Debug)]
pub enum ProcessCollectorError {
    NoMemory,
    ContextRegisterError,
    NtStatus(NtStatusError),
}
