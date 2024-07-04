#![no_std]

use core::panic::PanicInfo;

use wdk::{dbg_break, println};

#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::UNICODE_STRING;
use wdk_sys::{
    ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS,
    STATUS_UNSUCCESSFUL,
};
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf_std::kmalloc::TaggedObject;

pub mod collector;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

static CONTEXT_REGISTRY: FixedGlobalContextRegistry<10> = FixedGlobalContextRegistry::new();

struct TestDriverContext {
    a: u32,
    b: u32,
}

static DRIVER_CONTEXT: Context<TestDriverContext> = Context::uninit();

///# Safety
///
/// Driver entry point
///
///
#[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
pub unsafe extern "system" fn driver_entry(
    driver: &mut DRIVER_OBJECT,
    registry_path: PCUNICODE_STRING,
) -> NTSTATUS {
    dbg_break();

    match driver_main(driver, &*registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    driver.DriverUnload = Some(driver_unload);

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, || TestDriverContext { a: 10, b: 20 })?;

    Ok(())
}

pub unsafe extern "C" fn driver_unload(driver: *mut DRIVER_OBJECT) {
    dbg_break();

    CONTEXT_REGISTRY.drop_self();
}
