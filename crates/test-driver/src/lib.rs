#![no_std]

use core::{panic::PanicInfo, ptr::NonNull};

use collector::TestProcessCollector;
use wdk::{dbg_break, println};
#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::{ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS};
use wdrf::{
    driver::{DriverDispatch, DriverObject},
    framework::minifilter::{MinifilterFramework, MinifilterFrameworkBuilder},
    process::collector::ProcessRegistry,
};
use wdrf_std::{kmalloc::TaggedObject, string::ntunicode::NtUnicode};

pub mod collector;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

struct TestDriverContext {
    collector: ProcessRegistry<TestProcessCollector>,
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

    let framework = MinifilterFrameworkBuilder::new(
        NonNull::new(driver).unwrap(),
        NtUnicode::new(&*registry_path),
    )
    .unload(unload_callback)
    .build()
    .map_or(default, f);

    STATUS_SUCCESS
}

pub fn unload_callback(_framework: &mut MinifilterFramework) -> anyhow::Result<()> {
    Ok(())
}
