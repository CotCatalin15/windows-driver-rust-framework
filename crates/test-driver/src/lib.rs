#![no_std]

use core::panic::PanicInfo;
use core::time::Duration;

use maple::consumer::{get_global_registry, set_global_consumer};
use maple::{info, trace};

use wdk::{dbg_break, println};

#[cfg(not(test))]
use wdk_alloc::WDKAllocator;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL_ALLOCATOR: WDKAllocator = WDKAllocator;

use wdk_sys::fltmgr::{FLT_FILTER_UNLOAD_FLAGS, PFLT_PORT};
use wdk_sys::{
    ntddk::KeBugCheckEx, DRIVER_OBJECT, NTSTATUS, PCUNICODE_STRING, STATUS_SUCCESS,
    STATUS_UNSUCCESSFUL,
};
use wdk_sys::{OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, UNICODE_STRING};
use wdrf::context::{Context, ContextRegistry, FixedGlobalContextRegistry};
use wdrf::logger::dbgprint::DbgPrintLogger;
use wdrf::minifilter::communication::{FltCommunication, FltCommunicationBuilder};
use wdrf::minifilter::{FltFilter, FltRegistrationBuilder};
use wdrf::object::{ObjectAttribs, SecurityDescriptor};
use wdrf_std::slice::slice_from_raw_parts_mut_or_empty;
use wdrf_std::string::ntunicode::AsUnicodeString;
use wdrf_std::sync::arc::{Arc, ArcExt};
use wdrf_std::sys::event::KeEvent;
use wdrf_std::sys::WaitableObject;

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
    filter: Arc<FltFilter>,
    communication: Arc<FltCommunication>,
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
    dbg_break();

    match driver_main(driver, &*registry_path) {
        Ok(_) => STATUS_SUCCESS,
        Err(_) => STATUS_UNSUCCESSFUL,
    }
}

fn create_communication(filter: &Arc<FltFilter>) -> anyhow::Result<FltCommunication> {
    let name = widestring::u16cstr!("\\TESTPORT");
    let port_name = unsafe { name.as_unicode() };
    let descriptor = SecurityDescriptor::try_default_flt()?;

    let attribs = ObjectAttribs::new(
        &port_name,
        OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE,
        &descriptor,
    );

    FltCommunicationBuilder::new(filter.clone(), &attribs)
        .connect(Some(flt_comm_connection))
        .disconnect(Some(flt_comm_disconnect))
        .message(Some(flt_comm_notify))
        .build()
}

fn driver_main(
    driver: &mut DRIVER_OBJECT,
    registry_path: &'static UNICODE_STRING,
) -> anyhow::Result<()> {
    //let print_logge =
    //  DbgPrintLogger::new().map_err(|_| anyhow::Error::msg("Failed to create print logger"))?;

    let event = KeEvent::new();
    event.init(wdrf_std::sys::event::EventType::Synchronization);

    let status = event.wait_for(Duration::from_secs(5));

    println!("Wait status: {:#?}", status);

    event.signal();

    let status = event.wait();
    println!("Wait status: {:#?}", status);

    //LOGGER_CONTEXT.init(&CONTEXT_REGISTRY, move || print_logge)?;
    //set_global_consumer(LOGGER_CONTEXT.get());

    info!(name = "Driver entry", "Initializing driver");

    let registration = FltRegistrationBuilder::new()
        .unload(Some(minifilter_unload))
        .build()?;
    let filter = registration.register_filter(driver)?;
    let filter = Arc::try_create(filter)?;

    let comm = create_communication(&filter)?;
    let comm = Arc::try_create(comm)?;

    unsafe {
        filter.start_filtering()?;
    }

    DRIVER_CONTEXT.init(&CONTEXT_REGISTRY, || TestDriverContext {
        filter,
        communication: comm,
    })?;

    Ok(())
}

pub unsafe extern "C" fn minifilter_unload(_flags: FLT_FILTER_UNLOAD_FLAGS) -> NTSTATUS {
    dbg_break();

    info!(name = "Unload", "Unloading callback called");

    get_global_registry().disable_consumer();
    CONTEXT_REGISTRY.drop_self();
    STATUS_SUCCESS
}

unsafe extern "C" fn flt_comm_connection(
    client_port: PFLT_PORT,
    server_cookie: *mut core::ffi::c_void,
    connection_context: *mut core::ffi::c_void,
    size_of_context: u32,
    connection_port_cookie: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    dbg_break();

    STATUS_SUCCESS
}

unsafe extern "C" fn flt_comm_disconnect(client_cookie: *mut core::ffi::c_void) {
    dbg_break();
}

unsafe extern "C" fn flt_comm_notify(
    client_cookie: *mut core::ffi::c_void,
    input_buffer: *mut core::ffi::c_void,
    input_buffer_length: u32,
    output_buffer: *mut core::ffi::c_void,
    output_buffer_length: u32,
    return_output_buffer_length: *mut u32,
) -> NTSTATUS {
    dbg_break();

    let slice =
        slice_from_raw_parts_mut_or_empty(input_buffer as *mut u8, input_buffer_length as _);

    println!("Received: {:#?}", slice);

    STATUS_SUCCESS
}
