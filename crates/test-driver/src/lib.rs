#![no_std]

use core::panic::PanicInfo;

use wdk::{dbg_break, println};
#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::{ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING};
use wdrf::driver::{DriverDispatch, DriverObject};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

struct DriverContext {
    a: u32,
}

fn driver_unload(driver: &mut DriverObject) {
    dbg_break();

    let context = driver.get_context::<DriverContext>().unwrap();
    println!("Driver context a: {}", context.a);
}

pub fn driver_main(_: &mut DriverObject, dispatch: &mut DriverDispatch) -> anyhow::Result<()> {
    dispatch.set_context(DriverContext { a: 10 })?;

    dispatch.set_unload(driver_unload);
    Ok(())
}

///# Safety
///
/// Its safe its just for testing
///
#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    dbg_break();

    wdrf::Framework::run_entry(driver, registry_path, driver_main)
}
