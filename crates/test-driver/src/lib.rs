#![no_std]

use core::panic::PanicInfo;

use collector::TestCollector;
use flt_communication::{create_communication, FltCallbackImpl};
use maple::consumer::{get_global_registry, set_global_consumer};

use maple::info;
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::logger::DbgPrintLogger;
use wdrf::minifilter::communication::client_communication::FltClientCommunication;
use wdrf::minifilter::structs::IRP_MJ_OPERATION_END;
use wdrf::minifilter::{FltFilter, FltOperationRegistrationSlice, FltRegistrationBuilder};
use wdrf_std::constants::PoolFlags;
use wdrf_std::dbg_break;
use wdrf_std::kmalloc::{GlobalKernelAllocator, MemoryTag};
use windows_sys::Wdk::Foundation::DRIVER_OBJECT;
use windows_sys::Wdk::Storage::FileSystem::Minifilters::{
    FltGetRequestorProcessId, FLT_CALLBACK_DATA, FLT_OPERATION_REGISTRATION,
    FLT_POSTOP_CALLBACK_STATUS, FLT_POSTOP_FINISHED_PROCESSING, FLT_PREOP_CALLBACK_STATUS,
    FLT_PREOP_COMPLETE, FLT_RELATED_OBJECTS,
};
use windows_sys::Wdk::System::SystemServices::KeBugCheckEx;
use windows_sys::Win32::Foundation::{
    NTSTATUS, STATUS_SUCCESS, STATUS_UNSUCCESSFUL, UNICODE_STRING,
};

mod collector;
mod flt_communication;

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

#[allow(dead_code)]
unsafe extern "system" fn pre_op(
    data: *mut FLT_CALLBACK_DATA,
    _fltobjects: *const FLT_RELATED_OBJECTS,
    _completioncontext: *mut *mut core::ffi::c_void,
) -> FLT_PREOP_CALLBACK_STATUS {
    //SimRepGetIoOpenDriverRegistryKey
    FltGetRequestorProcessId(data);
    FLT_PREOP_COMPLETE
}

#[allow(dead_code)]
unsafe extern "system" fn post_op(
    _data: *mut FLT_CALLBACK_DATA,
    _fltobjects: *const FLT_RELATED_OBJECTS,
    _completioncontext: *const core::ffi::c_void,
    _flags: u32,
) -> FLT_POSTOP_CALLBACK_STATUS {
    FLT_POSTOP_FINISHED_PROCESSING
}

static FLT_OPS: Option<FltOperationRegistrationSlice<1>> = FltOperationRegistrationSlice::new([
    /*  FLT_OPERATION_REGISTRATION {
        MajorFunction: IRP_MJ_CREATE as _,
        Flags: 0,
        PreOperation: Some(pre_op),
        PostOperation: Some(post_op),
        Reserved1: core::ptr::null_mut(),
    },
    */
    FLT_OPERATION_REGISTRATION {
        MajorFunction: IRP_MJ_OPERATION_END as _,
        Flags: 0,
        PreOperation: None,
        PostOperation: None,
        Reserved1: core::ptr::null_mut(),
    },
]);

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
    dbg_break();

    init_logger();

    info!(name = "Driver entry", "Initializing driver");

    /*
        let registration = FltRegistrationBuilder::new()
            .unload(Some(minifilter_unload))
            .operations(FLT_OPS.as_ref().unwrap().get())
            .build()?;
        let filter = registration
            .register_filter(driver)
            .map_err(|_| anyhow::Error::msg("Failed to register filter"))?;

        let comm = create_communication(filter.clone())
            .map_err(|_| anyhow::Error::msg("Failed to create communication"))?;
    */

    driver.DriverUnload = Some(driver_unload);

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, move || TestDriverContext {
        collector: TestCollector::new(&CONTEXT_REGISTRY),
    })?;
    let context = DRIVER_CONTEXT.get();

    Ok(())
}

pub unsafe extern "system" fn driver_unload() {
    get_global_registry().disable_consumer();
    CONTEXT_REGISTRY.drop_self();
}

pub unsafe extern "system" fn minifilter_unload(_flags: u32) -> NTSTATUS {
    info!(name = "Unload", "Unloading callback called");

    get_global_registry().disable_consumer();
    CONTEXT_REGISTRY.drop_self();
    STATUS_SUCCESS
}
