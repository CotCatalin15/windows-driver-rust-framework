#![no_std]

use core::panic::PanicInfo;

use flt_communication::{create_communication, FltCallbackImpl};
use maple::consumer::{get_global_registry, set_global_consumer};
use maple::{info, trace};

use nt_string::unicode_string::NtUnicodeStr;
use wdk::{dbg_break, println};

#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::fltmgr::{FLT_FILTER_UNLOAD_FLAGS, PFLT_PORT};
use wdk_sys::ntddk::KeDelayExecutionThread;
use wdk_sys::_MODE::KernelMode;
use wdk_sys::{
    ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS,
    STATUS_UNSUCCESSFUL,
};
use wdk_sys::{LARGE_INTEGER, UNICODE_STRING};
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::logger::dbgprint::DbgPrintLogger;
use wdrf::minifilter::communication::client_communication::FltClientCommunication;
use wdrf::minifilter::{FltFilter, FltRegistrationBuilder};
use wdrf_std::slice::slice_from_raw_parts_mut_or_empty;
use wdrf_std::sync::arc::{Arc, ArcExt};
use widestring::Utf16Str;

mod collector;
mod flt_communication;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

static CONTEXT_REGISTRY: FixedGlobalContextRegistry<10> = FixedGlobalContextRegistry::new();

struct TestDriverContext {
    filter: Arc<FltFilter>,
    communication: FltClientCommunication<FltCallbackImpl>,
}

static DRIVER_CONTEXT: Context<TestDriverContext> = Context::uninit();
static LOGGER_CONTEXT: Context<DbgPrintLogger> = Context::uninit();

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
    match driver_main(driver, &*registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    //let print_logge =
    //  DbgPrintLogger::new().map_err(|_| anyhow::Error::msg("Failed to create print logger"))?;

    //LOGGER_CONTEXT.init(&CONTEXT_REGISTRY, move || print_logge)?;
    //set_global_consumer(LOGGER_CONTEXT.get());

    info!(name = "Driver entry", "Initializing driver");

    let registration = FltRegistrationBuilder::new()
        .unload(Some(minifilter_unload))
        .build()?;
    let filter = registration.register_filter(driver)?;
    let filter = Arc::try_create(filter)?;

    let comm = create_communication(filter.clone())?;

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, move || TestDriverContext {
        filter,
        communication: comm,
    })?;

    let context = DRIVER_CONTEXT.get();

    unsafe {
        context.filter.start_filtering();
    }

    Ok(())
}

pub unsafe extern "C" fn minifilter_unload(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    dbg_break();

    info!(name = "Unload", "Unloading callback called");

    get_global_registry().disable_consumer();
    CONTEXT_REGISTRY.drop_self();
    STATUS_SUCCESS
}
