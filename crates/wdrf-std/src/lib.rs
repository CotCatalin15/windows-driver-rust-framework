#![no_std]
#![feature(error_in_core)]
#![feature(allocator_api)]
#![feature(negative_impls)]
#![feature(effects)]
#![feature(arbitrary_self_types)]

use sealed::sealed;
use thiserror::Error;
use wdk_sys::{NTSTATUS, NT_SUCCESS};

extern crate alloc;

pub mod boxed;
pub mod fmt;
pub mod hashbrown;
pub mod io;
pub mod kmalloc;
pub mod object;
pub mod slice;
pub mod string;
pub mod sync;
pub mod thread;
pub mod traits;
pub mod vec;

pub mod sys;

#[derive(Error, Debug)]
pub enum NtStatusError {
    #[error("NtStatus code: {0:X}")]
    Status(i32),
}

pub type NtResult<T> = anyhow::Result<T, NtStatusError>;

#[sealed]
pub trait NtResultEx<T> {
    fn from_status<F: FnOnce() -> T>(status: NTSTATUS, f: F) -> NtResult<T>;
    fn from_status_err<E: FnOnce(), F: FnOnce() -> T>(status: NTSTATUS, e: E, f: F) -> NtResult<T>;
}

#[sealed]
impl<T> NtResultEx<T> for NtResult<T> {
    fn from_status<F: FnOnce() -> T>(status: NTSTATUS, f: F) -> NtResult<T> {
        if NT_SUCCESS(status) {
            Ok(f())
        } else {
            Err(NtStatusError::Status(status))
        }
    }

    fn from_status_err<E: FnOnce(), F: FnOnce() -> T>(status: NTSTATUS, e: E, f: F) -> NtResult<T> {
        if NT_SUCCESS(status) {
            Ok(f())
        } else {
            e();
            Err(NtStatusError::Status(status))
        }
    }
}
