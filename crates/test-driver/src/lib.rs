#![no_std]

use core::panic::PanicInfo;
use core::time::Duration;

use flt_communication::{create_communication, FltCallbackImpl};
use maple::consumer::get_global_registry;

use maple::info;
use wdk_sys::ntddk::KeBugCheckEx;
use wdk_sys::NTSTATUS;
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::minifilter::communication::client_communication::FltClientCommunication;
use wdrf::minifilter::{FltFilter, FltRegistrationBuilder};
use wdrf_std::constants::PoolFlags;
use wdrf_std::dbg_break;
use wdrf_std::kmalloc::{GlobalKernelAllocator, MemoryTag};
use wdrf_std::sync::arc::{Arc, ArcExt};
use wdrf_std::sys::event::EventType;
use wdrf_std::thread::{spawn, this_thread};
use windows_sys::Wdk::Foundation::DRIVER_OBJECT;
use windows_sys::Win32::Foundation::{STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING};

mod collector;
mod flt_communication;

#[global_allocator]
static KERNEL_GLOBAL_ALLOCATOR: GlobalKernelAllocator = GlobalKernelAllocator::new(
    MemoryTag::new_from_bytes(b"allc"),
    PoolFlags::POOL_FLAG_NON_PAGED,
);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        //println!("[PANIC] called: {:#?}", info);
        KeBugCheckEx(0x1234, 0, 0, 0, 0);
    }
}

static CONTEXT_REGISTRY: FixedGlobalContextRegistry<10> = FixedGlobalContextRegistry::new();

struct TestDriverContext {
    filter: Arc<FltFilter>,
    communication: FltClientCommunication<FltCallbackImpl>,
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
    registry_path: *const UNICODE_STRING,
) -> NTSTATUS {
    match driver_main(driver, &*registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

use wdrf_std::sync::event::Event;
use wdrf_std::sys::WaitableObject;

pub fn test() -> anyhow::Result<()> {
    let event = Event::try_create_arc(EventType::Notification, false)?;

    let s_event = event.clone();
    let th = spawn(move || {
        this_thread::delay_execution(Duration::from_secs(10));
        s_event.signal();
        this_thread::delay_execution(Duration::from_secs(5));
    })
    .map_err(|_| anyhow::Error::msg("Failed to create thread"))?;

    event.wait();

    th.join();

    Ok(())
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    dbg_break();
    //let print_logge =
    //  DbgPrintLogger::new().map_err(|_| anyhow::Error::msg("Failed to create print logger"))?;

    //LOGGER_CONTEXT.init(&CONTEXT_REGISTRY, move || print_logge)?;
    //set_global_consumer(LOGGER_CONTEXT.get());

    info!(name = "Driver entry", "Initializing driver");

    test()?;

    let registration = FltRegistrationBuilder::new()
        .unload(Some(minifilter_unload))
        .build()?;
    let filter = registration
        .register_filter(driver)
        .map_err(|_| anyhow::Error::msg("Failed to register filter"))?;

    let filter = Arc::try_create(filter)?;

    let comm = create_communication(filter.clone())
        .map_err(|_| anyhow::Error::msg("Failed to create communication"))?;

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, move || TestDriverContext {
        filter,
        communication: comm,
    })?;

    let context = DRIVER_CONTEXT.get();

    unsafe {
        context
            .filter
            .start_filtering()
            .map_err(|_| anyhow::Error::msg("Failed to start filtering"));
    }

    Ok(())
}

pub unsafe extern "system" fn minifilter_unload(_flags: u32) -> NTSTATUS {
    info!(name = "Unload", "Unloading callback called");

    get_global_registry().disable_consumer();
    CONTEXT_REGISTRY.drop_self();
    STATUS_SUCCESS
}
