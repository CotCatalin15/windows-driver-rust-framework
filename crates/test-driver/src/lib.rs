#![no_std]

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

use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;

use windows_sys::Wdk::Foundation::DRIVER_OBJECT;
use windows_sys::Wdk::System::SystemServices::{DbgPrint, KeBugCheckEx};
use windows_sys::Win32::Foundation::{
    NTSTATUS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

pub struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        0 as *mut u8
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        unreachable!(); // since we never allocate
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: Allocator = Allocator;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        //println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
        loop {}
    }
}

///# Safety
///
/// Driver entry point
///
///
#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: *const UNICODE_STRING,
) -> NTSTATUS {
    match driver_main(driver, &*registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    _registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    driver.DriverUnload = Some(driver_unload);

    unsafe {
        DbgPrint(c"Hello world".as_ptr() as _);
    }

    Ok(())
}

pub unsafe extern "system" fn driver_unload() {
    DbgPrint(c"Bye world".as_ptr() as _);
}
