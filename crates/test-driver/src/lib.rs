#![no_std]

use core::panic::PanicInfo;

use collector::TestCollector;
use flt_communication::create_communication;
use maple::consumer::{get_global_registry, set_global_consumer};

use maple::info;
use minifilter::MinifilterPreOperation;
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::logger::DbgPrintLogger;
use wdrf::minifilter::filter::framework::MinifilterFramework;
use wdrf::minifilter::filter::registration::{FltOperationEntry, FltOperationType};
use wdrf::minifilter::filter::{
    EmptyFltOperationsVisitor, FilterOperationVisitor, MinifilterFrameworkBuilder, UnloadStatus,
};
use wdrf_std::constants::PoolFlags;
use wdrf_std::kmalloc::{GlobalKernelAllocator, MemoryTag, TaggedObject};
use wdrf_std::{dbg_break, vec};
use windows_sys::Wdk::Foundation::DRIVER_OBJECT;
use windows_sys::Wdk::System::SystemServices::KeBugCheckEx;
use windows_sys::Win32::Foundation::{
    NTSTATUS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

mod collector;
mod flt_communication;
mod minifilter;

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
struct TestDriverContext {
    collector: TestCollector,
}

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

struct MinifilterUnload {}
impl TaggedObject for MinifilterUnload {}

impl FilterOperationVisitor for MinifilterUnload {
    fn unload(&self, mandatory: bool) -> wdrf::minifilter::filter::UnloadStatus {
        dbg_break();

        info!(name = "Unload", "Unloading callback called");

        get_global_registry().disable_consumer();
        CONTEXT_REGISTRY.drop_self();

        UnloadStatus::Unload
    }
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    _registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    dbg_break();

    init_logger();

    //driver.DriverUnload = Some(driver_unload);

    info!(name = "Driver entry", "Initializing driver");

    MinifilterFrameworkBuilder::new(MinifilterPreOperation {})
        .operations(&[FltOperationEntry::new(FltOperationType::Create, 0, false)])
        .post(EmptyFltOperationsVisitor {})
        .filter(MinifilterUnload {}, true)
        .build_and_register(&CONTEXT_REGISTRY, driver)?;

    let comm =
        create_communication().map_err(|_| anyhow::Error::msg("Failed to create communication"))?;

    driver.DriverUnload = Some(driver_unload);

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, move || TestDriverContext {
        collector: TestCollector::new(&CONTEXT_REGISTRY),
    })?;
    let context = DRIVER_CONTEXT.get();

    unsafe {
        MinifilterFramework::start_filtering().unwrap();
    }

    Ok(())
}

pub unsafe extern "system" fn driver_unload() {
    get_global_registry().disable_consumer();

    CONTEXT_REGISTRY.drop_self();
}
