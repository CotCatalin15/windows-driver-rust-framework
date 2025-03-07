#![no_std]
#![feature(allocator_api)]
#![feature(sync_unsafe_cell)]
#![feature(downcast_unchecked)]
#![feature(trait_alias)]
#![feature(box_into_inner)]

pub mod context;
pub mod logger;
pub mod macros;
pub mod mm;
pub mod process;

#[cfg(feature = "minifilter")]
pub mod minifilter;

// This is fine because we don't actually have any floating point instruction in
// our binary, thanks to our target defining soft-floats. fltused symbol is
// necessary due to LLVM being too eager to set it: it checks the LLVM IR for
// floating point instructions - even if soft-float is enabled!
#[allow(missing_docs)]
#[no_mangle]
pub static _fltused: () = ();

// FIXME: Is there any way to avoid this stub? See https://github.com/rust-lang/rust/issues/101134
#[allow(missing_docs)]
#[allow(clippy::missing_const_for_fn)] // const extern is not yet supported: https://github.com/rust-lang/rust/issues/64926
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3() -> i32 {
    0
}
