#![no_std]
#![feature(allocator_api)]
#![feature(negative_impls)]
#![feature(arbitrary_self_types)]
#![feature(slice_ptr_get)]

use sealed::sealed;
use thiserror::Error;
use windows_sys::Win32::Foundation::NTSTATUS;

extern crate alloc;

pub mod aligned;
pub mod boxed;
pub mod collections;
pub mod constants;
pub mod fmt;
pub mod hashbrown;
pub mod io;
pub mod kmalloc;
pub mod object;
pub mod slice;
pub mod structs;
pub mod sync;
pub mod thread;
pub mod time;
pub mod traits;
pub mod vec;

pub mod sys;

#[inline(always)]
pub fn nt_success(status: NTSTATUS) -> bool {
    status >= 0
}

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
        if nt_success(status) {
            Ok(f())
        } else {
            Err(NtStatusError::Status(status))
        }
    }

    fn from_status_err<E: FnOnce(), F: FnOnce() -> T>(status: NTSTATUS, e: E, f: F) -> NtResult<T> {
        if nt_success(status) {
            Ok(f())
        } else {
            e();
            Err(NtStatusError::Status(status))
        }
    }
}

/// Trigger a breakpoint in debugger via architecture-specific inline assembly.
///
/// Implementations derived from details outlined in [MSVC `__debugbreak` intrinsic documentation](https://learn.microsoft.com/en-us/cpp/intrinsics/debugbreak?view=msvc-170#remarks)
///
/// # Panics
/// Will Panic if called on an unsupported architecture
pub fn dbg_break() {
    // SAFETY: Abides all rules outlined in https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly
    unsafe {
        #[cfg(target_arch = "aarch64")]
        {
            core::arch::asm!("brk #0xF000");
            return;
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            core::arch::asm!("int 3");
            return;
        }
    }

    #[allow(unreachable_code)] // Code is not dead because of conditional compilation
    {
        panic!("dbg_break function called from unsupported architecture");
    }
}
