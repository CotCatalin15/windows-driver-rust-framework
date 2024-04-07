#![no_std]
#![feature(error_in_core)]
#![feature(allocator_api)]
#![feature(negative_impls)]

extern crate alloc;

pub mod boxed;
pub mod kmalloc;
pub mod sync;
mod sys;
pub mod traits;
pub mod vec;
