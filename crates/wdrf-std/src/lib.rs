#![no_std]
#![feature(error_in_core)]
#![feature(allocator_api)]
#![feature(negative_impls)]

use thiserror::Error;
use wdk_sys::{NTSTATUS, NT_SUCCESS};

extern crate alloc;

pub mod boxed;
pub mod hashbrown;
pub mod kmalloc;
pub mod sync;
pub mod traits;
pub mod vec;

mod sys;

#[derive(Error, Debug)]
#[error("NtStatus error {code}")]
pub struct NtStatusError {
    code: NTSTATUS,
}

pub type Result = anyhow::Result<(), NtStatusError>;

pub trait NtResultEx {
    fn from_status(status: NTSTATUS) -> Result {
        if NT_SUCCESS(status) {
            Ok(())
        } else {
            Err(NtStatusError { code: status })
        }
    }
}

impl NtResultEx for Result {}
impl NtResultEx for NTSTATUS {}
