#![no_std]

use core::panic::PanicInfo;

use maple::consumer::{get_global_registry, set_global_consumer};

use maple::info;
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::logger::DbgPrintLogger;
use wdrf::minifilter::filter::framework::MinifilterFramework;
use wdrf::minifilter::filter::registration::{FltOperationEntry, FltOperationType};
use wdrf_std::constants::PoolFlags;
use wdrf_std::kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject};
use wdrf_std::{dbg_break, vec};
use windows_sys::Wdk::Foundation::DRIVER_OBJECT;
use windows_sys::Wdk::System::SystemServices::KeBugCheckEx;
use windows_sys::Win32::Foundation::{
    CONTEXT_E_OLDREF, NTSTATUS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

use wdrf::minifilter::filter::builder::*;
use wdrf::minifilter::filter::*;

#[global_allocator]
static KERNEL_GLOBAL_ALLOCATOR: GlobalKernelAllocator = GlobalKernelAllocator::new(
    MemoryTag::new_from_bytes(b"allc"),
    PoolFlags::POOL_FLAG_NON_PAGED,
);

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        //println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
        loop {}
    }
}

static CONTEXT_REGISTRY: FixedGlobalContextRegistry<10> = FixedGlobalContextRegistry::new();

#[allow(dead_code)]
struct TestDriverContext {}

static DRIVER_CONTEXT: Context<TestDriverContext> = Context::uninit();

//#[no_mangle]
//static WdfMinimumVersionRequired: u32 = 33;

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

static LOGGER_CONTEXT: Context<DbgPrintLogger> = Context::uninit();

fn init_logger() {
    let logger = DbgPrintLogger::new();
    if logger.is_err() {
        return;
    }

    let logger = logger.unwrap();

    LOGGER_CONTEXT
        .init(&CONTEXT_REGISTRY, move || logger)
        .expect("Failed to init logger");

    set_global_consumer(LOGGER_CONTEXT.get());
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    _registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    driver.DriverUnload = Some(driver_unload);
    Ok(())
}

pub unsafe extern "system" fn driver_unload() {}
